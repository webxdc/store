//! Parser for commands sent to the bot

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(author = None, version = None, about = None, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    Download { file: String },
    Open
}
