use std::path::{Path, PathBuf};

use anyhow::Context;

/// 設定ファイルを生成するユースケース
pub struct InitConfig;

/// 書き出すデフォルト設定テンプレート
const CONFIG_TEMPLATE: &str = r#"# wtxr 設定ファイル
# https://github.com/your-org/wtxr

[worktree]
# ワークツリーを配置するルートディレクトリ（リポジトリルートからの相対パス）
# root_dir = ".wtxr/worktrees"

# 新規ブランチ作成時のデフォルトベースブランチ
# default_base_branch = "main"

[copy]
# ワークツリー作成時にコピーするファイルの glob パターン
# patterns = ["*.env", ".envrc"]

# 明示的なファイルマッピング（from → to でリネームも可能）
# [[copy.files]]
# from = ".env.example"
# to = ".env"

[hooks]
# ワークツリー作成前に実行するコマンド（リポジトリルートで実行）
# pre_create = []

# ワークツリー作成後に実行するコマンド（ワークツリー内で実行）
# post_create = ["npm install"]

# ファイルコピー後に実行するコマンド（ワークツリー内で実行）
# post_copy = []
"#;

impl InitConfig {
    /// 設定ファイルを生成する
    ///
    /// - `path`: 書き出すファイルパス
    /// - `force`: true のとき既存ファイルを上書き
    pub fn execute(path: &Path, force: bool) -> anyhow::Result<()> {
        if path.exists() && !force {
            anyhow::bail!(
                "config file already exists: {}\n\
                 Use --force to overwrite.",
                path.display()
            );
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(path, CONFIG_TEMPLATE)
            .with_context(|| format!("failed to write config: {}", path.display()))?;

        Ok(())
    }
}

/// 生成先パスを返す（テスト用に公開）
pub fn config_path(global: bool, repo_root: Option<&Path>) -> anyhow::Result<PathBuf> {
    if global {
        dirs::config_dir()
            .map(|d| d.join("wtxr").join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("cannot determine config directory"))
    } else {
        let root = repo_root.ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;
        Ok(root.join(".wtxr").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn creates_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".wtxr/config.toml");

        InitConfig::execute(&path, false).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[worktree]"));
        assert!(content.contains("[copy]"));
        assert!(content.contains("[hooks]"));
    }

    #[test]
    fn fails_if_file_exists_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        fs::write(&path, "existing").unwrap();

        let err = InitConfig::execute(&path, false).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn overwrites_with_force() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        fs::write(&path, "old content").unwrap();

        InitConfig::execute(&path, true).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[worktree]"));
    }

    #[test]
    fn creates_parent_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("deeply/nested/config.toml");

        InitConfig::execute(&path, false).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn local_path_is_under_wtxr() {
        let tmp = tempfile::tempdir().unwrap();
        let path = config_path(false, Some(tmp.path())).unwrap();
        assert_eq!(path, tmp.path().join(".wtxr/config.toml"));
    }
}
