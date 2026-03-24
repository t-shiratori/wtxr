use std::path::Path;

/// フック実行のインターフェース
pub trait HookRunner {
    /// コマンドリストを順番に実行する
    ///
    /// - `commands`: 実行するシェルコマンドのリスト
    /// - `cwd`: コマンドを実行するディレクトリ
    fn run(&self, commands: &[String], cwd: &Path) -> anyhow::Result<()>;
}
