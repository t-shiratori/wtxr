use clap::{Parser, Subcommand};

pub mod add;
pub mod init;
pub mod list;
pub mod remove;

#[derive(Parser)]
#[command(
    name = "wtxr",
    version,
    about = "A git worktree management CLI tool",
    long_about = None,
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new worktree
    Add(add::AddArgs),
    /// List all worktrees
    List(list::ListArgs),
    /// Remove worktree(s)
    Remove(remove::RemoveArgs),
    /// Initialize config file
    Init(init::InitArgs),
}
