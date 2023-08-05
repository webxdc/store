//! Command line interface.

use clap::{Parser, Subcommand};

/// Command line argument parser.
#[derive(Parser, Debug)]
#[command()]
pub struct BotCli {
    #[allow(clippy::missing_docs_in_private_items)]
    #[command(subcommand)]
    pub action: BotActions,
}

/// Command line subcommands.
#[derive(Subcommand, Debug)]
pub enum BotActions {
    /// Start the bot.
    Start,
    /// Import xdcs.
    Import {
        /// Path from which files should be imported.
        path: String,
    },
    /// Show the 1:1-invite QR code.
    ShowQr,
    /// Show the bots version.
    Version,
}
