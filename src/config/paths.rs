use std::path::{Path, PathBuf};

use super::types::Config;

const DEFAULT_WORKTREES_DIR: &str = ".wtxr/worktrees";

/// ワークツリーを配置するルートディレクトリを解決する
pub fn resolve_worktrees_dir(repo_root: &Path, cfg: &Config) -> PathBuf {
    let root_dir = &cfg.worktree.root_dir;
    if root_dir.is_empty() {
        repo_root.join(DEFAULT_WORKTREES_DIR)
    } else {
        repo_root.join(root_dir)
    }
}

/// ブランチ名からワークツリーのパスを解決する
///
/// ブランチ名のスラッシュはディレクトリセパレータとして扱う。
/// 例: "feature/foo" → "<worktrees_dir>/feature/foo"
pub fn resolve_worktree_path(repo_root: &Path, cfg: &Config, branch: &str) -> PathBuf {
    let worktrees_dir = resolve_worktrees_dir(repo_root, cfg);
    worktrees_dir.join(branch)
}

/// ユーザー入力のパスを絶対パスに解決する
///
/// 解決規則:
/// 1. 絶対パス → そのまま返す
/// 2. 存在するパス → カレントディレクトリからの絶対パスに変換
/// 3. 存在しない相対パス → ワークツリーディレクトリ配下として解決
pub fn resolve_input_worktree_path(
    repo_root: &Path,
    cfg: &Config,
    input: &str,
) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(input);

    if path.is_absolute() {
        return Ok(path);
    }

    if path.exists() {
        return Ok(path.canonicalize()?);
    }

    Ok(resolve_worktrees_dir(repo_root, cfg).join(input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Config;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from("/repo")
    }

    #[test]
    fn default_worktrees_dir() {
        let cfg = Config::default();
        let result = resolve_worktrees_dir(&repo_root(), &cfg);
        assert_eq!(result, PathBuf::from("/repo/.wtxr/worktrees"));
    }

    #[test]
    fn custom_root_dir() {
        let mut cfg = Config::default();
        cfg.worktree.root_dir = "custom/dir".to_string();
        let result = resolve_worktrees_dir(&repo_root(), &cfg);
        assert_eq!(result, PathBuf::from("/repo/custom/dir"));
    }

    #[test]
    fn worktree_path_from_branch() {
        let cfg = Config::default();
        let result = resolve_worktree_path(&repo_root(), &cfg, "feature/foo");
        assert_eq!(
            result,
            PathBuf::from("/repo/.wtxr/worktrees/feature/foo")
        );
    }

    #[test]
    fn absolute_input_path_returned_as_is() {
        let cfg = Config::default();
        let result =
            resolve_input_worktree_path(&repo_root(), &cfg, "/absolute/path").unwrap();
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn non_existing_relative_path_resolved_to_worktrees_dir() {
        let cfg = Config::default();
        let result =
            resolve_input_worktree_path(&repo_root(), &cfg, "feature/bar").unwrap();
        assert_eq!(
            result,
            PathBuf::from("/repo/.wtxr/worktrees/feature/bar")
        );
    }
}
