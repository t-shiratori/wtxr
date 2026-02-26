use std::path::{Path, PathBuf};

use crate::domain::worktree::{AddOptions, Worktree};

/// Git リポジトリ操作のインターフェース（Go の interface に相当）
///
/// テスト時はモック実装に差し替え可能。
pub trait GitRepository {
    /// ワークツリーを追加する
    fn add_worktree(&self, path: &Path, opts: &AddOptions) -> anyhow::Result<()>;

    /// ワークツリーを削除する
    fn remove_worktree(&self, path: &Path, force: bool) -> anyhow::Result<()>;

    /// ブランチを削除する
    fn delete_branch(&self, branch: &str, force: bool) -> anyhow::Result<()>;

    /// 全ワークツリーの一覧を返す
    fn list_worktrees(&self) -> anyhow::Result<Vec<Worktree>>;

    /// 指定パスのワークツリーのブランチ名を返す
    fn branch_from_worktree(&self, path: &Path) -> anyhow::Result<Option<String>>;

    /// リポジトリのルートパスを返す
    fn repo_root(&self) -> anyhow::Result<PathBuf>;
}
