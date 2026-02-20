use serde::{Deserialize, Serialize};

/// 設定ファイル全体の構造体
///
/// TOML 例:
/// ```toml
/// [worktree]
/// root_dir = ".wtxr/worktrees"
/// default_base_branch = "main"
///
/// [copy]
/// patterns = ["*.env"]
/// [[copy.files]]
/// from = ".env.example"
/// to = ".env"
///
/// [hooks]
/// pre_create = ["echo pre"]
/// post_create = ["echo post"]
/// post_copy = ["echo copied"]
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub worktree: WorktreeConfig,
    pub copy: CopyConfig,
    pub hooks: HooksConfig,
}

/// ワークツリーのパス設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorktreeConfig {
    /// ワークツリーを配置するルートディレクトリ（リポジトリルートからの相対パス）
    /// 空の場合は ".wtxr/worktrees" をデフォルトとして使用
    pub root_dir: String,

    /// デフォルトのベースブランチ
    pub default_base_branch: String,
}

/// ファイルコピーの設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CopyConfig {
    /// glob パターンによるコピー対象ファイル（例: "*.env", "config/*.yaml"）
    pub patterns: Vec<String>,

    /// 明示的なファイルマッピング（リネームも可能）
    pub files: Vec<CopyFile>,
}

/// from → to のファイルマッピング
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFile {
    pub from: String,
    pub to: String,
}

/// フック設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HooksConfig {
    /// ワークツリー作成前に実行するコマンドリスト
    pub pre_create: Vec<String>,

    /// ワークツリー作成後に実行するコマンドリスト
    pub post_create: Vec<String>,

    /// ファイルコピー後に実行するコマンドリスト
    pub post_copy: Vec<String>,
}
