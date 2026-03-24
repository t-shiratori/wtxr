use std::path::Path;
use std::process::Command;

use anyhow::Context;

use crate::port::hook::HookRunner;

/// フック実行のアダプター実装（シェル経由でコマンドを実行）
pub struct ShellHookRunner;

impl ShellHookRunner {
    pub fn new() -> Self {
        Self
    }
}

impl HookRunner for ShellHookRunner {
    fn run(&self, commands: &[String], cwd: &Path) -> anyhow::Result<()> {
        for cmd in commands {
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(cwd)
                .status()
                .with_context(|| format!("failed to spawn hook: {cmd}"))?;

            if !status.success() {
                let code = status.code().unwrap_or(-1);
                anyhow::bail!("hook failed (exit {code}): {cmd}");
            }
        }
        Ok(())
    }
}
