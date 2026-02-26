use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context};

use crate::domain::worktree::{AddOptions, Worktree};
use crate::port::git::GitRepository;

/// `git` コマンドを呼び出す GitRepository の実装
pub struct GitAdapter;

impl GitAdapter {
    pub fn new() -> Self {
        Self
    }

    /// git コマンドを実行し、stdout を文字列で返す
    fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(args)
            .output()
            .with_context(|| format!("failed to run: git {}", args.join(" ")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            bail!("git {} failed: {}", args.join(" "), stderr)
        }
    }
}

impl GitRepository for GitAdapter {
    fn add_worktree(&self, path: &Path, opts: &AddOptions) -> anyhow::Result<()> {
        let path_str = path.to_string_lossy();
        let mut args = vec!["worktree", "add"];

        if opts.create_branch {
            args.push("-b");
            args.push(&opts.branch);
        }

        args.push(&path_str);

        // ベースブランチが指定されていれば末尾に追加
        let from = opts.from.as_deref().unwrap_or("");
        if !from.is_empty() {
            args.push(from);
        } else if !opts.create_branch {
            // 新規ブランチを作らない場合、branch 名をそのまま渡す
            args.push(&opts.branch);
        }

        self.run(&args)?;
        Ok(())
    }

    fn remove_worktree(&self, path: &Path, force: bool) -> anyhow::Result<()> {
        let path_str = path.to_string_lossy();
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(&path_str);
        self.run(&args)?;
        Ok(())
    }

    fn delete_branch(&self, branch: &str, force: bool) -> anyhow::Result<()> {
        let flag = if force { "-D" } else { "-d" };
        self.run(&["branch", flag, branch])?;
        Ok(())
    }

    fn list_worktrees(&self) -> anyhow::Result<Vec<Worktree>> {
        let output = self.run(&["worktree", "list", "--porcelain"])?;
        parse_worktree_list(&output)
    }

    fn branch_from_worktree(&self, path: &Path) -> anyhow::Result<Option<String>> {
        let worktrees = self.list_worktrees()?;
        let found = worktrees
            .into_iter()
            .find(|w| w.path == *path)
            .map(|w| w.branch);
        Ok(found.flatten())
    }

    fn repo_root(&self) -> anyhow::Result<PathBuf> {
        let output = self.run(&["rev-parse", "--show-toplevel"])?;
        Ok(PathBuf::from(output.trim()))
    }
}

/// `git worktree list --porcelain` の出力をパースする
///
/// 出力形式:
/// ```text
/// worktree /path/to/main
/// HEAD abc1234
/// branch refs/heads/main
///
/// worktree /path/to/feature
/// HEAD def5678
/// branch refs/heads/feature/foo
///
/// worktree /path/to/detached
/// HEAD ghi9012
/// detached
/// ```
fn parse_worktree_list(output: &str) -> anyhow::Result<Vec<Worktree>> {
    let mut worktrees = Vec::new();
    let mut path: Option<PathBuf> = None;
    let mut commit: Option<String> = None;
    let mut branch: Option<String> = None;

    for line in output.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            // 前のエントリが完成していれば追加
            if let (Some(p), Some(c)) = (path.take(), commit.take()) {
                worktrees.push(Worktree::new(p, branch.take(), c));
            }
            path = Some(PathBuf::from(p));
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            commit = Some(h.to_string());
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            branch = Some(b.to_string());
        }
        // "detached" 行は branch を None のままにする
    }

    // 最後のエントリを追加
    if let (Some(p), Some(c)) = (path, commit) {
        worktrees.push(Worktree::new(p, branch, c));
    }

    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORCELAIN_OUTPUT: &str = "\
worktree /repo
HEAD abc1234567890
branch refs/heads/main

worktree /repo/.wtxr/worktrees/feature/foo
HEAD def5678901234
branch refs/heads/feature/foo

worktree /repo/.wtxr/worktrees/detached
HEAD ghi9012345678
detached

";

    #[test]
    fn parse_normal_worktrees() {
        let result = parse_worktree_list(PORCELAIN_OUTPUT).unwrap();
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].path, PathBuf::from("/repo"));
        assert_eq!(result[0].branch, Some("main".to_string()));
        assert_eq!(result[0].commit, "abc1234567890");

        assert_eq!(
            result[1].path,
            PathBuf::from("/repo/.wtxr/worktrees/feature/foo")
        );
        assert_eq!(result[1].branch, Some("feature/foo".to_string()));
        assert_eq!(result[1].commit, "def5678901234");
    }

    #[test]
    fn parse_detached_head() {
        let result = parse_worktree_list(PORCELAIN_OUTPUT).unwrap();
        assert_eq!(result[2].branch, None);
        assert_eq!(result[2].commit, "ghi9012345678");
    }

    #[test]
    fn parse_empty_output() {
        let result = parse_worktree_list("").unwrap();
        assert!(result.is_empty());
    }
}
