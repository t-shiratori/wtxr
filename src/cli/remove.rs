use clap::Args;

use crate::adapter::git::GitAdapter;
use crate::config::load_config;
use crate::config::paths::resolve_input_worktree_path;
use crate::domain::worktree::RemoveOptions;
use crate::port::git::GitRepository;
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

    // 削除対象のパス一覧を解決する
    // 引数なしなら TUI で対話的に選択
    let paths = if args.worktrees.is_empty() {
        let all = ListWorktrees::new(&git).execute()?;
        // メインのワークツリー（最初のエントリ）は除外する
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

    for path in paths {
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
