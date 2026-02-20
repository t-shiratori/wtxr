use clap::Args;

#[derive(Args)]
pub struct ListArgs {}

pub fn run(_args: &ListArgs) -> anyhow::Result<()> {
    println!("not implemented");
    Ok(())
}
