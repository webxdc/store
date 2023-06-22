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
use clap::Parser;
use cli::{BotActions, BotCli};
use log::info;
use tokio::signal;
use utils::get_version;

use crate::request_handlers::AppInfo;

const DB_URL: &str = "sqlite://bot-db/bot.db";
const DC_DB_PATH: &str = "./deltachat.db";
const GENESIS_QR: &str = "./bot-data/genesis_invite_qr.png";
const INVITE_QR: &str = "./bot-data/1o1_invite_qr.png";
const SHOP_XDC: &str = "./bot-data/store.xdc";
const SUBMIT_HELPER_XDC: &str = "./bot-data/submit-helper.xdc";
const REVIEW_HELPER_XDC: &str = "./bot-data/review-helper.xdc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import { path, keep_files } => {
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
                println!("No xdcs to add in {}", path)
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
                    let mut app_info = AppInfo::from_xdc(file).await?;
                    app_info.active = true;
                    app_info.author_name = "xdcstore".to_string();
                    app_info.author_email = "xdcstore@testrun.org".to_string();

                    let missing = app_info.generate_missing_list();
                    if missing.is_empty() {
                        let mut new_path = PathBuf::from("./bot-data/xdcs");
                        new_path.push(file.file_name().context("Direntry has no filename")?);

                        if keep_files.unwrap_or_default() {
                            fs::copy(file, &new_path).with_context(|| {
                                format!(
                                    "failed to copy {} to {}",
                                    file.display(),
                                    new_path.display()
                                )
                            })?;
                        } else {
                            fs::rename(file, &new_path).with_context(|| {
                                format!(
                                    "failed to move {} to {}",
                                    file.display(),
                                    new_path.display()
                                )
                            })?;
                        }
                        app_info.xdc_blob_dir = Some(new_path);
                        db::create_app_info(&mut *bot.get_db_connection().await?, &mut app_info)
                            .await?;
                        println!("Added {:?}({}) to apps", file, app_info.name);
                    } else {
                        println!(
                            "The app {} is missing some data: {:?}",
                            file.as_os_str().to_str().context("Can't convert to str")?,
                            missing
                        )
                    }
                }
            }
        }
        BotActions::ShowQr => {
            let bot = Bot::new().await.context("failed to create bot")?;
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
        BotActions::Version => print!("{}", get_version().await?),
        BotActions::Start => {
            let mut bot = Bot::new().await.context("failed to create bot")?;
            bot.start().await;
            signal::ctrl_c().await?;
        }
    }
    Ok(())
}
