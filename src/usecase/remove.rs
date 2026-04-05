use std::path::Path;

use crate::adapter::filesystem::remove_empty_parents;
use crate::config::paths::resolve_worktrees_dir;
use crate::config::Config;
use crate::domain::worktree::RemoveOptions;
use crate::port::git::GitRepository;

pub struct RemoveWorktree<'a> {
    git: &'a dyn GitRepository,
    cfg: &'a Config,
    repo_root: &'a Path,
}

impl<'a> RemoveWorktree<'a> {
    pub fn new(git: &'a dyn GitRepository, cfg: &'a Config, repo_root: &'a Path) -> Self {
        Self { git, cfg, repo_root }
    }

    /// dry-run 用: 実際の操作は行わず実行予定の操作一覧を返す
    pub fn plan(&self, opts: &RemoveOptions) -> super::DryRunPlan {
        let mut rows: Vec<(String, String)> = Vec::new();

        let mut git_args = format!("remove {}", opts.path.display());
        if opts.force {
            git_args.push_str(" --force");
        }
        rows.push(("git worktree".to_string(), git_args));

        if opts.delete_branch {
            rows.push((
                "git branch".to_string(),
                format!("-d <branch of {}>", opts.path.display()),
            ));
        }

        super::DryRunPlan {
            title: format!("[dry-run] wtxr remove {}", opts.path.display()),
            rows,
        }
    }

    pub fn execute(&self, opts: &RemoveOptions) -> anyhow::Result<()> {
        // ブランチ削除が必要なら先にブランチ名を取得しておく
        let branch = if opts.delete_branch {
            self.git.branch_from_worktree(&opts.path)?
        } else {
            None
        };

        // ワークツリーを削除
        self.git.remove_worktree(&opts.path, opts.force)?;

        // ブランチも削除
        if let Some(b) = branch {
            self.git.delete_branch(&b, opts.force)?;
        }

        // 空になった親ディレクトリを片付ける
        let worktrees_dir = resolve_worktrees_dir(self.repo_root, self.cfg);
        remove_empty_parents(&opts.path, &worktrees_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    use crate::domain::worktree::{AddOptions, RemoveOptions, Worktree};
    use crate::port::git::GitRepository;

    struct MockGit {
        removed: RefCell<Vec<(PathBuf, bool)>>,
        deleted_branches: RefCell<Vec<(String, bool)>>,
        branch_for_path: Option<String>,
    }

    impl MockGit {
        fn new(branch_for_path: Option<&str>) -> Self {
            Self {
                removed: RefCell::new(vec![]),
                deleted_branches: RefCell::new(vec![]),
                branch_for_path: branch_for_path.map(String::from),
            }
        }
    }

    impl GitRepository for MockGit {
        fn add_worktree(&self, _path: &Path, _opts: &AddOptions) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn remove_worktree(&self, path: &Path, force: bool) -> anyhow::Result<()> {
            self.removed.borrow_mut().push((path.to_path_buf(), force));
            Ok(())
        }
        fn delete_branch(&self, branch: &str, force: bool) -> anyhow::Result<()> {
            self.deleted_branches
                .borrow_mut()
                .push((branch.to_string(), force));
            Ok(())
        }
        fn list_worktrees(&self) -> anyhow::Result<Vec<Worktree>> {
            Ok(vec![])
        }
        fn branch_from_worktree(&self, _path: &Path) -> anyhow::Result<Option<String>> {
            Ok(self.branch_for_path.clone())
        }
        fn repo_root(&self) -> anyhow::Result<PathBuf> {
            Ok(PathBuf::from("/repo"))
        }
    }

    #[test]
    fn removes_worktree_without_branch() {
        let git = MockGit::new(None);
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = RemoveWorktree::new(&git, &cfg, &repo_root);

        let opts = RemoveOptions {
            path: PathBuf::from("/repo/.wtxr/worktrees/feature/foo"),
            delete_branch: false,
            force: false,
        };
        uc.execute(&opts).unwrap();

        assert_eq!(git.removed.borrow().len(), 1);
        assert_eq!(git.deleted_branches.borrow().len(), 0);
    }

    #[test]
    fn removes_worktree_and_branch() {
        let git = MockGit::new(Some("feature/foo"));
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = RemoveWorktree::new(&git, &cfg, &repo_root);

        let opts = RemoveOptions {
            path: PathBuf::from("/repo/.wtxr/worktrees/feature/foo"),
            delete_branch: true,
            force: false,
        };
        uc.execute(&opts).unwrap();

        assert_eq!(git.removed.borrow().len(), 1);
        let branches = git.deleted_branches.borrow();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].0, "feature/foo");
    }

    #[test]
    fn force_flag_is_passed_through() {
        let git = MockGit::new(Some("main"));
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = RemoveWorktree::new(&git, &cfg, &repo_root);

        let opts = RemoveOptions {
            path: PathBuf::from("/repo/.wtxr/worktrees/main"),
            delete_branch: true,
            force: true,
        };
        uc.execute(&opts).unwrap();

        let removed = git.removed.borrow();
        assert!(removed[0].1, "force should be true for remove_worktree");
        let branches = git.deleted_branches.borrow();
        assert!(branches[0].1, "force should be true for delete_branch");
    }
}
