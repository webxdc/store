use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command()]
pub struct Genesis {
    #[command(subcommand)]
    pub join: GroupName,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BotGroup {
    Genesis,
    Reviewee,
    Tester,
}

#[derive(Subcommand, Debug)]
pub enum GroupName {
    Join {
        #[arg(short, long)]
        name: BotGroup,
    },
}
