use std::path::Path;

use crate::config::Config;
use crate::config::paths::resolve_worktree_path;
use crate::domain::worktree::AddOptions;
use crate::port::filesystem::FileSystem;
use crate::port::git::GitRepository;
use crate::port::hook::HookRunner;

pub struct AddWorktree<'a> {
    git: &'a dyn GitRepository,
    fs: &'a dyn FileSystem,
    hooks: &'a dyn HookRunner,
    cfg: &'a Config,
    repo_root: &'a Path,
}

impl<'a> AddWorktree<'a> {
    pub fn new(
        git: &'a dyn GitRepository,
        fs: &'a dyn FileSystem,
        hooks: &'a dyn HookRunner,
        cfg: &'a Config,
        repo_root: &'a Path,
    ) -> Self {
        Self { git, fs, hooks, cfg, repo_root }
    }

    /// dry-run 用: 実際の操作は行わず実行予定の操作一覧を返す
    pub fn plan(&self, opts: &AddOptions) -> super::DryRunPlan {
        let worktree_path = resolve_worktree_path(self.repo_root, self.cfg, &opts.branch);
        let path_str = worktree_path.display().to_string();
        let mut rows: Vec<(String, String)> = Vec::new();

        // git worktree add
        let git_args = if opts.create_branch {
            if let Some(from) = &opts.from {
                format!("add -b {} {} {}", opts.branch, path_str, from)
            } else {
                format!("add -b {} {}", opts.branch, path_str)
            }
        } else {
            format!("add {} {}", path_str, opts.branch)
        };
        rows.push(("git worktree".to_string(), git_args));

        // pre_create フック
        for cmd in &self.cfg.hooks.pre_create {
            rows.push((
                "hook pre".to_string(),
                format!("{}  (cwd: {})", cmd, self.repo_root.display()),
            ));
        }

        // post_create フック
        for cmd in &self.cfg.hooks.post_create {
            rows.push((
                "hook post".to_string(),
                format!("{}  (cwd: {})", cmd, path_str),
            ));
        }

        // copy + post_copy フック
        let has_copy = !self.cfg.copy.patterns.is_empty() || !self.cfg.copy.files.is_empty();
        if has_copy {
            for pattern in &self.cfg.copy.patterns {
                rows.push((
                    "copy (glob)".to_string(),
                    format!("{} → {}/*", pattern, path_str),
                ));
            }
            for file in &self.cfg.copy.files {
                rows.push((
                    "copy (file)".to_string(),
                    format!("{} → {}/{}", file.from, path_str, file.to),
                ));
            }
            for cmd in &self.cfg.hooks.post_copy {
                rows.push((
                    "hook post-copy".to_string(),
                    format!("{}  (cwd: {})", cmd, path_str),
                ));
            }
        }

        super::DryRunPlan {
            title: format!("[dry-run] wtxr add {}", opts.branch),
            rows,
        }
    }

    pub fn execute(&self, opts: &AddOptions) -> anyhow::Result<()> {
        let worktree_path = resolve_worktree_path(self.repo_root, self.cfg, &opts.branch);

        // pre_create フック（リポジトリルートで実行）
        self.hooks.run(&self.cfg.hooks.pre_create, self.repo_root)?;

        // ワークツリーを作成
        self.git.add_worktree(&worktree_path, opts)?;

        // post_create フック（ワークツリーで実行）
        self.hooks.run(&self.cfg.hooks.post_create, &worktree_path)?;

        // 設定に基づいてファイルをコピー
        let has_copy = !self.cfg.copy.patterns.is_empty() || !self.cfg.copy.files.is_empty();
        if has_copy {
            self.fs.copy_files(self.repo_root, &worktree_path, &self.cfg.copy)?;

            // post_copy フック（ワークツリーで実行）
            self.hooks.run(&self.cfg.hooks.post_copy, &worktree_path)?;
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
    use crate::port::hook::HookRunner;

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

    struct MockHooks {
        // `ran` は run の過去形。「実行済みのフック呼び出し履歴」を表す
        ran: RefCell<Vec<(Vec<String>, PathBuf)>>,
    }

    impl MockHooks {
        fn new() -> Self {
            Self { ran: RefCell::new(vec![]) }
        }
    }

    impl HookRunner for MockHooks {
        fn run(&self, commands: &[String], cwd: &Path) -> anyhow::Result<()> {
            if !commands.is_empty() {
                self.ran.borrow_mut().push((commands.to_vec(), cwd.to_path_buf()));
            }
            Ok(())
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
        let hooks = MockHooks::new();
        let cfg = Config::default();
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        let added = git.added.borrow();
        assert_eq!(added[0].0, PathBuf::from("/repo/.wtxr/worktrees/feature/foo"));
    }

    #[test]
    fn file_copy_is_skipped_when_config_is_empty() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let hooks = MockHooks::new();
        let cfg = Config::default(); // copy が空
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        assert!(fs.copied.borrow().is_empty(), "copy_files should not be called");
    }

    #[test]
    fn file_copy_is_called_when_patterns_exist() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let hooks = MockHooks::new();
        let mut cfg = Config::default();
        cfg.copy.patterns = vec!["*.env".to_string()];
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/bar")).unwrap();

        let copied = fs.copied.borrow();
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0].1, PathBuf::from("/repo/.wtxr/worktrees/feature/bar"));
    }

    #[test]
    fn pre_create_hook_runs_before_git() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let hooks = MockHooks::new();
        let mut cfg = Config::default();
        cfg.hooks.pre_create = vec!["echo pre".to_string()];
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        let ran = hooks.ran.borrow();
        assert_eq!(ran[0].0, vec!["echo pre"]);
        assert_eq!(ran[0].1, repo_root); // リポジトリルートで実行
    }

    #[test]
    fn post_create_and_post_copy_hooks_run_in_worktree() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let hooks = MockHooks::new();
        let mut cfg = Config::default();
        cfg.hooks.post_create = vec!["echo post-create".to_string()];
        cfg.hooks.post_copy = vec!["echo post-copy".to_string()];
        cfg.copy.patterns = vec!["*.env".to_string()];
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/bar")).unwrap();

        let ran = hooks.ran.borrow();
        let worktree_path = PathBuf::from("/repo/.wtxr/worktrees/feature/bar");
        assert_eq!(ran[0].0, vec!["echo post-create"]);
        assert_eq!(ran[0].1, worktree_path);
        assert_eq!(ran[1].0, vec!["echo post-copy"]);
        assert_eq!(ran[1].1, worktree_path);
    }

    #[test]
    fn post_copy_hook_is_skipped_when_no_copy_config() {
        let git = MockGit::new();
        let fs = MockFs::new();
        let hooks = MockHooks::new();
        let mut cfg = Config::default();
        cfg.hooks.post_copy = vec!["echo post-copy".to_string()];
        // copy 設定なし → post_copy も実行されない
        let repo_root = PathBuf::from("/repo");
        let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

        uc.execute(&default_opts("feature/foo")).unwrap();

        assert!(hooks.ran.borrow().is_empty());
    }
}
