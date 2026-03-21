use std::path::Path;

use anyhow::Context;

use crate::config::types::{CopyConfig, CopyFile};
use crate::port::filesystem::FileSystem;

/// ファイルシステム操作のアダプター実装
pub struct FsAdapter;

impl FsAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for FsAdapter {
    fn copy_files(
        &self,
        src_root: &Path,
        dst_root: &Path,
        cfg: &CopyConfig,
    ) -> anyhow::Result<Vec<CopyFile>> {
        let mut copied = Vec::new();

        // 1. glob パターンによるコピー
        for pattern in &cfg.patterns {
            let full_pattern = src_root.join(pattern).to_string_lossy().into_owned();
            for entry in glob::glob(&full_pattern)? {
                let src = entry?;
                if !src.is_file() {
                    continue; // ディレクトリはスキップ
                }
                // src_root からの相対パスを保ちつつ dst_root 配下にコピー
                let rel = src.strip_prefix(src_root)?;
                let dst = dst_root.join(rel);
                copy_file(&src, &dst)?;
                copied.push(CopyFile {
                    from: rel.to_string_lossy().into_owned(),
                    to: rel.to_string_lossy().into_owned(),
                });
            }
        }

        // 2. 明示的ファイルマッピングによるコピー
        for file in &cfg.files {
            let src = src_root.join(&file.from);
            if !src.exists() {
                continue; // 存在しなければスキップ
            }
            let dst = dst_root.join(&file.to);
            copy_file(&src, &dst)?;
            copied.push(file.clone());
        }

        Ok(copied)
    }
}

/// ファイルを src から dst にコピーする（親ディレクトリを自動作成）
fn copy_file(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }
    std::fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} → {}", src.display(), dst.display()))?;
    Ok(())
}

/// ワークツリー削除後に残った空の親ディレクトリを再帰的に削除する
///
/// `stop_dir` に到達したら停止する（そこ自体は削除しない）。
pub fn remove_empty_parents(dir: &Path, stop_dir: &Path) -> anyhow::Result<()> {
    let mut current = dir;

    loop {
        if current == stop_dir || !current.starts_with(stop_dir) {
            break;
        }

        if !current.exists() {
            // すでに削除済みなら親へ
        } else {
            let is_empty = current.read_dir()?.next().is_none();
            if !is_empty {
                break;
            }
            std::fs::remove_dir(current)?;
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_dirs(base: &Path, rel_path: &str) -> PathBuf {
        let target = base.join(rel_path);
        fs::create_dir_all(&target).unwrap();
        target
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    // --- remove_empty_parents のテスト ---

    #[test]
    fn removes_empty_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf = setup_dirs(&worktrees, "feature/my-task");

        fs::remove_dir(&leaf).unwrap();
        remove_empty_parents(&leaf, &worktrees).unwrap();

        assert!(!worktrees.join("feature").exists());
        assert!(worktrees.exists());
    }

    #[test]
    fn stops_when_parent_is_not_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf1 = setup_dirs(&worktrees, "feature/task-1");
        let _leaf2 = setup_dirs(&worktrees, "feature/task-2");

        fs::remove_dir(&leaf1).unwrap();
        remove_empty_parents(&leaf1, &worktrees).unwrap();

        assert!(worktrees.join("feature").exists());
    }

    #[test]
    fn does_not_remove_stop_dir_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf = setup_dirs(&worktrees, "solo");

        fs::remove_dir(&leaf).unwrap();
        remove_empty_parents(&leaf, &worktrees).unwrap();

        assert!(worktrees.exists());
    }

    // --- FsAdapter::copy_files のテスト ---

    #[test]
    fn copy_by_glob_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        write_file(&src.join(".env"), "SECRET=foo");
        write_file(&src.join("other.txt"), "hello");

        let cfg = CopyConfig {
            patterns: vec!["*.env".to_string()],
            files: vec![],
        };

        let adapter = FsAdapter::new();
        let copied = adapter.copy_files(&src, &dst, &cfg).unwrap();

        assert_eq!(copied.len(), 1);
        assert!(dst.join(".env").exists());
        assert!(!dst.join("other.txt").exists());
    }

    #[test]
    fn copy_explicit_file_with_rename() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        write_file(&src.join(".env.example"), "SECRET=example");

        let cfg = CopyConfig {
            patterns: vec![],
            files: vec![CopyFile {
                from: ".env.example".to_string(),
                to: ".env".to_string(),
            }],
        };

        let adapter = FsAdapter::new();
        let copied = adapter.copy_files(&src, &dst, &cfg).unwrap();

        assert_eq!(copied.len(), 1);
        assert!(dst.join(".env").exists());
        assert!(!dst.join(".env.example").exists());
    }

    #[test]
    fn skips_nonexistent_explicit_file() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();

        let cfg = CopyConfig {
            patterns: vec![],
            files: vec![CopyFile {
                from: "nonexistent".to_string(),
                to: "nonexistent".to_string(),
            }],
        };

        let adapter = FsAdapter::new();
        let copied = adapter.copy_files(&src, &dst, &cfg).unwrap();
        assert!(copied.is_empty());
    }
}
