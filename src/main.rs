#![warn(clippy::all, clippy::indexing_slicing, clippy::unwrap_used)]
mod bot;
mod bot_commands;
mod cli;
mod db;
mod messages;
mod request_handlers;
mod utils;

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use bot::Bot;
use build_script_file_gen::include_file_str;
use clap::Parser;
use cli::{BotActions, BotCli};
use log::info;
use tokio::signal;

use crate::request_handlers::AppInfo;

const DB_URL: &str = "sqlite://bot-db/bot.db";
const DC_DB_PATH: &str = "./deltachat.db";
const GENESIS_QR: &str = "./bot-data/genesis_invite_qr.png";
const INVITE_QR: &str = "./bot-data/1o1_invite_qr.png";
const SHOP_XDC: &str = "./bot-data/store.xdc";
const SUBMIT_HELPER_XDC: &str = "./bot-data/submit-helper.xdc";
const REVIEW_HELPER_XDC: &str = "./bot-data/review-helper.xdc";
const VERSION: &str = include_file_str!("VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import { path } => {
            let bot = Bot::new().await.context("failed to create bot")?;
            let path = path.as_deref().unwrap_or("import/");
            info!("Importing webxdcs from {path}");
            let dir_entry = std::fs::read_dir(path).context("failed to read dir")?;

            let files: Vec<_> = dir_entry
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|e| e.is_file())
                .collect();

            if files.is_empty() {
                eprintln!("No xdcs to add in {}", path);
                return Ok(());
            }

            if !PathBuf::from("./bot-data/xdcs")
                .try_exists()
                .unwrap_or_default()
            {
                fs::create_dir("./bot-data/xdcs")
                    .context("failed to create ./bot-data/xdcs directory")?;
            }

            for file in &files {
                if file
                    .file_name()
                    .and_then(|a| a.to_str())
                    .context("Can't get filename for imported file")?
                    .ends_with(".xdc")
                {
                    match AppInfo::from_xdc(file).await {
                        Ok(mut app_info) => {
                            app_info.active = true;
                            app_info.submitter_uri = Some("xdcstore".to_string());

                            let mut new_path = PathBuf::from("./bot-data/xdcs");
                            new_path.push(file.file_name().context("Direntry has no filename")?);

                            fs::copy(file, &new_path).with_context(|| {
                                format!(
                                    "failed to copy {} to {}",
                                    file.display(),
                                    new_path.display()
                                )
                            })?;

                            app_info.xdc_blob_dir = new_path;
                            db::create_app_info(
                                &mut *bot.get_db_connection().await?,
                                &mut app_info,
                            )
                            .await?;
                            println!("Added {}({}) to apps", file.display(), app_info.name);
                        }
                        Err(e) => {
                            println!("Failed to import {:?} \n{}", file, e);
                        }
                    };
                }
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
            bot.start().await?;
            signal::ctrl_c().await?;
        }
    }
    Ok(())
}
