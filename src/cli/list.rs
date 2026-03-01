use clap::Args;

use crate::adapter::git::GitAdapter;
use crate::domain::worktree::Worktree;
use crate::usecase::list::ListWorktrees;

#[derive(Args)]
pub struct ListArgs {}

pub fn run(_args: &ListArgs) -> anyhow::Result<()> {
    let git = GitAdapter::new();
    let uc = ListWorktrees::new(&git);
    let worktrees = uc.execute()?;

    print_worktrees(&worktrees);
    Ok(())
}

/// ワークツリー一覧を表形式で出力する
///
/// 出力例:
/// ```
/// /repo                               main              abc1234
/// /repo/.wtxr/worktrees/feature/foo   feature/foo       def5678
/// ```
fn print_worktrees(worktrees: &[Worktree]) {
    if worktrees.is_empty() {
        return;
    }

    // 各列の最大幅を計算してパディング揃え
    let path_width = worktrees
        .iter()
        .map(|w| w.path.to_string_lossy().len())
        .max()
        .unwrap_or(0);

    let branch_width = worktrees
        .iter()
        .map(|w| w.branch_display().len())
        .max()
        .unwrap_or(0);

    for wt in worktrees {
        let path = wt.path.to_string_lossy();
        let branch = wt.branch_display();
        let commit = &wt.commit[..7.min(wt.commit.len())];

        println!(
            "{:<path_width$}    {:<branch_width$}    {}",
            path,
            branch,
            commit,
            path_width = path_width,
            branch_width = branch_width,
        );
    }
}
