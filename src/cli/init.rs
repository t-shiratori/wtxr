use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    /// Overwrite existing config file
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Create global config (~/.config/wtxr/config.toml)
    #[arg(long = "global")]
    pub global: bool,
}

pub fn run(_args: &InitArgs) -> anyhow::Result<()> {
    println!("not implemented");
    Ok(())
}
