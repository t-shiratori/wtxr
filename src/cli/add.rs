use clap::Args;

use crate::adapter::filesystem::FsAdapter;
use crate::adapter::git::GitAdapter;
use crate::adapter::hook::ShellHookRunner;
use crate::config::load_config;
use crate::domain::worktree::AddOptions;
use crate::logger;
use crate::spinner::Spinner;
use crate::usecase::add::AddWorktree;

#[derive(Args)]
pub struct AddArgs {
    /// Branch name to checkout
    pub branch: String,

    /// Create a new branch
    #[arg(short = 'b', long = "create-branch")]
    pub create_branch: bool,

    /// Base branch to create from
    #[arg(long = "from")]
    pub from: Option<String>,

    /// Preview what would happen without making changes
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

pub fn run(args: &AddArgs) -> anyhow::Result<()> {
    let git = GitAdapter::new();
    let repo_root = git.repo_root()?;
    let cfg = load_config(&repo_root)?;

    let opts = AddOptions {
        branch: args.branch.clone(),
        create_branch: args.create_branch,
        from: args.from.clone(),
    };

    let fs = FsAdapter::new();
    let hooks = ShellHookRunner::new();
    let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);

    if args.dry_run {
        uc.plan(&opts).print();
        return Ok(());
    }

    logger::verbose(&format!("repo root: {}", repo_root.display()));
    logger::verbose(&format!("branch: {}", opts.branch));

    let spinner = Spinner::new(&format!("Adding worktree for '{}'…", opts.branch));

    match uc.execute(&opts) {
        Ok(()) => spinner.success(&format!("Added worktree for '{}'", opts.branch)),
        Err(e) => {
            spinner.fail(&format!("Failed to add worktree for '{}'", opts.branch));
            return Err(e);
        }
    }

    Ok(())
}
