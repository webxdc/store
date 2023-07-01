use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command()]
pub struct BotCli {
    #[command(subcommand)]
    pub action: BotActions,
}

#[derive(Subcommand, Debug)]
pub enum BotActions {
    /// Start the bot.
    Start,
    /// Import xdcs.
    Import {
        /// Path from which files should be imported.
        path: String,
    },
    /// Show the 1:1-invite and genesis-invite qr code.
    ShowQr,
    /// Show the bots version.
    Version,
}
