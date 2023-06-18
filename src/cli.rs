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
        /// Path from which files should be imported. Default is '/import'.
        path: Option<String>,
        /// Boolean flag whether xdcs should be removed. Default is false.
        keep_files: Option<bool>,
    },
    /// Show the 1:1-invite and genesis-invite qr code.
    ShowQr,
}
