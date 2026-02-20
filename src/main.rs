mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add(args) => cli::add::run(args),
        Commands::List(args) => cli::list::run(args),
        Commands::Remove(args) => cli::remove::run(args),
        Commands::Init(args) => cli::init::run(args),
    }
}
