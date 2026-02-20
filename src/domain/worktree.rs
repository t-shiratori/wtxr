use std::path::PathBuf;

/// Git ワークツリーを表すドメインエンティティ
#[derive(Debug, Clone)]
pub struct Worktree {
    /// ワークツリーの絶対パス
    pub path: PathBuf,
    /// チェックアウトされているブランチ名（detached HEAD の場合は None）
    pub branch: Option<String>,
    /// HEAD コミットハッシュ
    pub commit: String,
}

impl Worktree {
    pub fn new(path: PathBuf, branch: Option<String>, commit: String) -> Self {
        Self {
            path,
            branch,
            commit,
        }
    }

    /// ブランチ名を表示用の文字列として返す（detached HEAD は短縮ハッシュ）
    pub fn branch_display(&self) -> String {
        match &self.branch {
            Some(b) => b.clone(),
            None => format!("({})", &self.commit[..7.min(self.commit.len())]),
        }
    }
}

/// `wtxr add` で使うオプション
#[derive(Debug, Clone)]
pub struct AddOptions {
    /// チェックアウトするブランチ名
    pub branch: String,
    /// 新規ブランチを作成するか
    pub create_branch: bool,
    /// 起点となるブランチ（None の場合は HEAD）
    pub from: Option<String>,
}

/// `wtxr remove` で使うオプション
#[derive(Debug, Clone)]
pub struct RemoveOptions {
    /// ワークツリーの絶対パス
    pub path: PathBuf,
    /// ブランチも削除するか
    pub delete_branch: bool,
    /// 強制削除するか
    pub force: bool,
}
