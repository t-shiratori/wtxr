use clap::Args;

use crate::adapter::git::GitAdapter;
use crate::config::load_config;
use crate::config::paths::resolve_input_worktree_path;
use crate::domain::worktree::RemoveOptions;
use crate::logger;
use crate::port::git::GitRepository;
use crate::spinner::Spinner;
use crate::tui::select::select_worktrees;
use crate::usecase::list::ListWorktrees;
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
    let git = GitAdapter::new();
    let repo_root = git.repo_root()?;
    let cfg = load_config(&repo_root)?;

    let paths = if args.worktrees.is_empty() {
        let all = ListWorktrees::new(&git).execute()?;
        let candidates: Vec<_> = all.into_iter().skip(1).collect();

        if candidates.is_empty() {
            println!("No worktrees to remove.");
            return Ok(());
        }

        match select_worktrees(candidates)? {
            Some(selected) if !selected.is_empty() => {
                selected.into_iter().map(|wt| wt.path).collect()
            }
            _ => {
                println!("No worktrees selected.");
                return Ok(());
            }
        }
    } else {
        args.worktrees
            .iter()
            .map(|input| resolve_input_worktree_path(&repo_root, &cfg, input))
            .collect::<anyhow::Result<Vec<_>>>()?
    };

    for path in &paths {
        let opts = RemoveOptions {
            path: path.to_path_buf(),
            delete_branch: args.delete_branch,
            force: args.force,
        };
        let uc = RemoveWorktree::new(&git, &cfg, &repo_root);

        if args.dry_run {
            uc.plan(&opts).print();
            continue;
        }

        logger::verbose(&format!("removing: {}", path.display()));

        let path_str = path.to_string_lossy().into_owned();
        let label = path.file_name().and_then(|n| n.to_str()).unwrap_or(&path_str);
        let spinner = Spinner::new(&format!("Removing '{label}'…"));

        match uc.execute(&opts) {
            Ok(()) => spinner.success(&format!("Removed '{}'", path.display())),
            Err(e) => {
                spinner.fail(&format!("Failed to remove '{}'", path.display()));
                return Err(e);
            }
        }
    }

    Ok(())
}
