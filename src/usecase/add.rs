use std::path::Path;

use crate::config::Config;
use crate::config::paths::resolve_worktree_path;
use crate::domain::worktree::AddOptions;
use crate::port::git::GitRepository;

pub struct AddWorktree<'a> {
    git: &'a dyn GitRepository,
    cfg: &'a Config,
    repo_root: &'a Path,
}

impl<'a> AddWorktree<'a> {
    pub fn new(git: &'a dyn GitRepository, cfg: &'a Config, repo_root: &'a Path) -> Self {
        Self { git, cfg, repo_root }
    }

    pub fn execute(&self, opts: &AddOptions) -> anyhow::Result<()> {
        let worktree_path = resolve_worktree_path(self.repo_root, self.cfg, &opts.branch);
        self.git.add_worktree(&worktree_path, opts)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    use crate::domain::worktree::{AddOptions, Worktree};
    use crate::port::git::GitRepository;

    /// テスト用モック
    struct MockGit {
        added: RefCell<Vec<(PathBuf, AddOptions)>>,
    }

    impl MockGit {
        fn new() -> Self {
            Self { added: RefCell::new(vec![]) }
        }
    }

    impl GitRepository for MockGit {
        fn add_worktree(&self, path: &Path, opts: &AddOptions) -> anyhow::Result<()> {
            self.added.borrow_mut().push((path.to_path_buf(), opts.clone()));
            Ok(())
        }
        fn remove_worktree(&self, _path: &Path, _force: bool) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn delete_branch(&self, _branch: &str, _force: bool) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn list_worktrees(&self) -> anyhow::Result<Vec<Worktree>> {
            Ok(vec![])
        }
        fn branch_from_worktree(&self, _path: &Path) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
        fn repo_root(&self) -> anyhow::Result<PathBuf> {
            Ok(PathBuf::from("/repo"))
        }
    }

    fn default_opts(branch: &str) -> AddOptions {
        AddOptions {
            branch: branch.to_string(),
            create_branch: false,
            from: None,
        }
    }

    #[test]
    fn worktree_path_is_resolved_from_branch() {
        let git = MockGit::new();
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &cfg, &repo_root);

        let opts = default_opts("feature/foo");
        uc.execute(&opts).unwrap();

        let added = git.added.borrow();
        assert_eq!(added.len(), 1);
        assert_eq!(
            added[0].0,
            PathBuf::from("/repo/.wtxr/worktrees/feature/foo")
        );
    }

    #[test]
    fn create_branch_flag_is_passed_through() {
        let git = MockGit::new();
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &cfg, &repo_root);

        let opts = AddOptions {
            branch: "new-branch".to_string(),
            create_branch: true,
            from: Some("main".to_string()),
        };
        uc.execute(&opts).unwrap();

        let added = git.added.borrow();
        assert!(added[0].1.create_branch);
        assert_eq!(added[0].1.from, Some("main".to_string()));
    }
}
