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
    /// Import xdcs from './import/'.
    Import,
    /// Show the 1:1-invite and genesis-invite qr code.
    ShowQr,
}
