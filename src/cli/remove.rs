use clap::Args;

#[derive(Args)]
pub struct RemoveArgs {
    /// Worktree paths or branch names to remove (interactive TUI if omitted)
    pub worktrees: Vec<String>,

    /// Also delete the branch
    #[arg(short = 'b', long = "branch")]
    pub delete_branch: bool,

    /// Force removal even if the worktree has uncommitted changes
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Preview what would happen without making changes
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

pub fn run(_args: &RemoveArgs) -> anyhow::Result<()> {
    println!("not implemented");
    Ok(())
}
