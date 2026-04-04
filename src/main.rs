mod adapter;
mod cli;
mod config;
mod domain;
mod logger;
mod port;
mod spinner;
mod tui;
mod usecase;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    logger::set_verbose(cli.verbose);

    match &cli.command {
        Commands::Add(args) => cli::add::run(args),
        Commands::List(args) => cli::list::run(args),
        Commands::Remove(args) => cli::remove::run(args),
        Commands::Init(args) => cli::init::run(args),
    }
}
