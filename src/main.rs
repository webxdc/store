#![warn(clippy::all, clippy::indexing_slicing, clippy::unwrap_used)]
mod bot;
mod cli;
mod db;
mod import;
mod messages;
mod request_handlers;
mod utils;
use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use bot::Bot;
use build_script_file_gen::include_file_str;
use clap::Parser;
use cli::{BotActions, BotCli};
use tokio::signal;
use utils::{project_dirs, AddType};

const GENESIS_QR: &str = "genesis_invite_qr.png";
const INVITE_QR: &str = "1o1_invite_qr.png";
const VERSION: &str = include_file_str!("VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import { path } => {
            let path = PathBuf::from(path);
            let bot = Bot::new().await.context("failed to create bot")?;
            let xdcs_dir = project_dirs()?.config_dir().to_path_buf().join("xdcs");
            create_dir_all(&xdcs_dir)?;

            if path.is_file() {
                match import::import_one(
                    path.as_path(),
                    &xdcs_dir,
                    &mut *bot.get_db_connection().await?,
                )
                .await?
                {
                    AddType::Added => println!("Added {}", path.display()),
                    AddType::Updated => println!("Updated {}", path.display()),
                    AddType::Ignored => println!("Ignored {}", path.display()),
                }
            } else if path.is_dir() {
                import::import_many(
                    path.as_path(),
                    xdcs_dir,
                    &mut *bot.get_db_connection().await?,
                )
                .await?;
            } else {
                eprintln!("{} is not a file or directory", path.display());
            }
        }
        BotActions::ShowQr => {
            let bot = Bot::new().await.context("Failed to create bot")?;
            match db::get_config(&mut *bot.get_db_connection().await?).await {
                Ok(config) => {
                    println!("Genesis invite qr:");
                    qr2term::print_qr(config.genesis_qr)?;
                    println!("Bot invite qr:");
                    qr2term::print_qr(config.invite_qr)?;
                }
                Err(_) => println!("Bot not configured yet, start the bot first."),
            }
        }
        BotActions::Version => print!("{}", VERSION),
        BotActions::Start => {
            let mut bot = Bot::new().await.context("Failed to create bot")?;
            bot.start().await;
            signal::ctrl_c().await?;
        }
    }
    Ok(())
}
