use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command()]
pub struct BotCli {
    #[command(subcommand)]
    pub action: BotActions,
}

#[derive(Subcommand, Debug)]
pub enum BotActions {
    Start,
    Import,
}
