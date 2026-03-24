use clap::Args;

use crate::adapter::filesystem::FsAdapter;
use crate::adapter::git::GitAdapter;
use crate::adapter::hook::ShellHookRunner;
use crate::config::load_config;
use crate::domain::worktree::AddOptions;
use crate::port::git::GitRepository;
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

    if args.dry_run {
        print_dry_run(&opts, &cfg, &repo_root);
        return Ok(());
    }

    let fs = FsAdapter::new();
    let hooks = ShellHookRunner::new();
    let uc = AddWorktree::new(&git, &fs, &hooks, &cfg, &repo_root);
    uc.execute(&opts)?;

    println!("Added worktree for '{}'", opts.branch);
    Ok(())
}

fn print_dry_run(
    opts: &AddOptions,
    cfg: &crate::config::Config,
    repo_root: &std::path::Path,
) {
    use crate::config::paths::resolve_worktree_path;

    let path = resolve_worktree_path(repo_root, cfg, &opts.branch);
    println!("[dry-run] add worktree");
    println!("  branch : {}", opts.branch);
    println!("  path   : {}", path.display());
    if let Some(from) = &opts.from {
        println!("  from   : {}", from);
    }
    if opts.create_branch {
        println!("  create : true");
    }
}
