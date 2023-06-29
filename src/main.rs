#![warn(clippy::all, clippy::indexing_slicing, clippy::unwrap_used)]
mod bot;
mod bot_commands;
mod cli;
mod db;
mod import;
mod messages;
mod request_handlers;
mod utils;
use std::{fs::create_dir_all, path::PathBuf};

use anyhow::{Context as _, Result};
use bot::Bot;
use build_script_file_gen::include_file_str;
use clap::Parser;
use cli::{BotActions, BotCli};
use directories::ProjectDirs;
use tokio::signal;

const GENESIS_QR: &str = "genesis_invite_qr.png";
const INVITE_QR: &str = "1o1_invite_qr.png";
const STORE_XDC: &str = "store.xdc";
const SUBMIT_HELPER_XDC: &str = "submit-helper.xdc";
const REVIEW_HELPER_XDC: &str = "review-helper.xdc";
const VERSION: &str = include_file_str!("VERSION");

pub(crate) fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "XDC Store").context("cannot determine home directory")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import { path } => {
            let path = PathBuf::from(path.as_deref().unwrap_or("."));
            let bot = Bot::new().await.context("failed to create bot")?;
            let xdcs_dir = project_dirs()?.config_dir().to_path_buf().join("xdcs");
            create_dir_all(&xdcs_dir)?;

            if path.is_file() {
                import::import_one(
                    path.as_path(),
                    &xdcs_dir,
                    &mut *bot.get_db_connection().await?,
                )
                .await?;
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
                    println!("Genisis invite qr:");
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
