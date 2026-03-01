use crate::domain::worktree::Worktree;
use crate::port::git::GitRepository;

pub struct ListWorktrees<'a> {
    git: &'a dyn GitRepository,
}

impl<'a> ListWorktrees<'a> {
    pub fn new(git: &'a dyn GitRepository) -> Self {
        Self { git }
    }

    pub fn execute(&self) -> anyhow::Result<Vec<Worktree>> {
        self.git.list_worktrees()
    }
}
