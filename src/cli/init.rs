use clap::Args;

use crate::adapter::git::GitAdapter;
use crate::port::git::GitRepository;
use crate::usecase::init::{config_path, InitConfig};

#[derive(Args)]
pub struct InitArgs {
    /// Overwrite existing config file
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Create global config (~/.config/wtxr/config.toml)
    #[arg(long = "global")]
    pub global: bool,
}

pub fn run(args: &InitArgs) -> anyhow::Result<()> {
    // グローバルの場合はリポジトリルート不要
    let repo_root = if args.global {
        None
    } else {
        let git = GitAdapter::new();
        Some(git.repo_root()?)
    };

    let path = config_path(args.global, repo_root.as_deref())?;

    InitConfig::execute(&path, args.force)?;

    println!("Created config: {}", path.display());
    Ok(())
}
