use clap::Args;

use crate::adapter::git::GitAdapter;
use crate::config::load_config;
use crate::config::paths::resolve_input_worktree_path;
use crate::domain::worktree::RemoveOptions;
use crate::port::git::GitRepository;
use crate::usecase::remove::RemoveWorktree;

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

pub fn run(args: &RemoveArgs) -> anyhow::Result<()> {
    if args.worktrees.is_empty() {
        // 引数なし → Step 7 で TUI を実装予定
        eprintln!("No worktree specified. Interactive selection coming soon (Step 7).");
        return Ok(());
    }

    let git = GitAdapter::new();
    let repo_root = git.repo_root()?;
    let cfg = load_config(&repo_root)?;

    for input in &args.worktrees {
        let path = resolve_input_worktree_path(&repo_root, &cfg, input)?;

        if args.dry_run {
            print_dry_run(&path, args);
            continue;
        }

        let opts = RemoveOptions {
            path: path.clone(),
            delete_branch: args.delete_branch,
            force: args.force,
        };

        let uc = RemoveWorktree::new(&git, &cfg, &repo_root);
        uc.execute(&opts)?;
        println!("Removed worktree '{}'", path.display());
    }

    Ok(())
}

fn print_dry_run(path: &std::path::Path, args: &RemoveArgs) {
    println!("[dry-run] remove worktree");
    println!("  path          : {}", path.display());
    println!("  delete-branch : {}", args.delete_branch);
    println!("  force         : {}", args.force);
}
