use clap::Args;

#[derive(Args)]
pub struct AddArgs {
    /// Branch name to checkout
    pub branch: String,

    /// Create a new branch
    #[arg(short = 'b', long = "create-branch")]
    pub create_branch: bool,

    /// Base branch to create from
    #[arg(long = "from")]
    pub from: Option<String>,

    /// Preview what would happen without making changes
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

pub fn run(_args: &AddArgs) -> anyhow::Result<()> {
    println!("not implemented");
    Ok(())
}
