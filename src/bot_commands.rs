use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command()]
pub struct Genesis {
    pub join: Option<BotGroup>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BotGroup {
    Publisher,
    Tester,
}

#[derive(Subcommand, Debug)]
pub enum GroupName {
    Join {
        #[arg(short, long)]
        name: BotGroup,
    },
}
