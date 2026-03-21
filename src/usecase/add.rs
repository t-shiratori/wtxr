use std::path::Path;

use crate::config::Config;
use crate::config::paths::resolve_worktree_path;
use crate::domain::worktree::AddOptions;
use crate::port::filesystem::FileSystem;
use crate::port::git::GitRepository;

pub struct AddWorktree<'a> {
    git: &'a dyn GitRepository,
    fs: &'a dyn FileSystem,
    cfg: &'a Config,
    repo_root: &'a Path,
}

impl<'a> AddWorktree<'a> {
    pub fn new(
        git: &'a dyn GitRepository,
        fs: &'a dyn FileSystem,
        cfg: &'a Config,
        repo_root: &'a Path,
    ) -> Self {
        Self { git, fs, cfg, repo_root }
    }

    pub fn execute(&self, opts: &AddOptions) -> anyhow::Result<()> {
        let worktree_path = resolve_worktree_path(self.repo_root, self.cfg, &opts.branch);

        // ワークツリーを作成
        self.git.add_worktree(&worktree_path, opts)?;

        // 設定に基づいてファイルをコピー
        if !self.cfg.copy.patterns.is_empty() || !self.cfg.copy.files.is_empty() {
            self.fs
                .copy_files(self.repo_root, &worktree_path, &self.cfg.copy)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    use crate::config::types::{CopyConfig, CopyFile};
    use crate::domain::worktree::{AddOptions, Worktree};
    use crate::port::filesystem::FileSystem;
    use crate::port::git::GitRepository;

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

    struct MockFs {
        copied: RefCell<Vec<(PathBuf, PathBuf)>>,
    }

    impl MockFs {
        fn new() -> Self {
            Self { copied: RefCell::new(vec![]) }
        }
    }

    impl FileSystem for MockFs {
        fn copy_files(
            &self,
            src_root: &Path,
            dst_root: &Path,
            _cfg: &CopyConfig,
        ) -> anyhow::Result<Vec<CopyFile>> {
            self.copied
                .borrow_mut()
                .push((src_root.to_path_buf(), dst_root.to_path_buf()));
            Ok(vec![])
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
        let fs = MockFs::new();
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        let added = git.added.borrow();
        assert_eq!(added[0].0, PathBuf::from("/repo/.wtxr/worktrees/feature/foo"));
    }

    #[test]
    fn file_copy_is_skipped_when_config_is_empty() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let cfg = Config::default(); // copy が空
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        assert!(fs.copied.borrow().is_empty(), "copy_files should not be called");
    }

    #[test]
    fn file_copy_is_called_when_patterns_exist() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let mut cfg = Config::default();
        cfg.copy.patterns = vec!["*.env".to_string()];
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &cfg, &repo_root);

        uc.execute(&default_opts("feature/bar")).unwrap();

        let copied = fs.copied.borrow();
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0].1, PathBuf::from("/repo/.wtxr/worktrees/feature/bar"));
    }
}
