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

use crate::request_handlers::AppInfo;

const DB_URL: &str = "sqlite://bot-db/bot.db";
const DC_DB_PATH: &str = "deltachat.db";
const GENESIS_QR: &str = "./bot-data/genesis_invite_qr.png";
const INVITE_QR: &str = "./bot-data/1o1_invite_qr.png";
const SHOP_XDC: &str = "./bot-data/appstore.xdc";
const SUBMIT_HELPER_XDC: &str = "./bot-data/submit-helper.xdc";
const REVIEW_HELPER_XDC: &str = "./bot-data/review-helper.xdc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = BotCli::parse();
    let mut bot = Bot::new().await.context("failed to create bot")?;

    match &cli.action {
        BotActions::Import => {
            info!("Importing webxdcs from 'import/'");
            let dir_entry = match std::fs::read_dir("import/") {
                Ok(dir) => dir,
                Err(_) => {
                    fs::create_dir("import/").ok();
                    fs::read_dir("import/")?
                }
            };

            let files: Vec<_> = dir_entry
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|e| e.is_file())
                .collect();

            if files.is_empty() {
                println!("No xdcs to add in ./import")
            }

            if !PathBuf::from("./bot-data/xdcs")
                .try_exists()
                .unwrap_or_default()
            {
                fs::create_dir("./bot-data/xdcs")?;
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
                    app_info.author_name = "Appstore bot".to_string();
                    app_info.author_email = "appstorebot@testrun.org".to_string();

                    let missing = app_info.generate_missing_list();
                    if missing.is_empty() {
                        let mut new_path = file
                            .parent()
                            .and_then(|a| a.parent())
                            .context("Path could not be constructed")?
                            .to_path_buf();

                        new_path.push("bot-data/xdcs");
                        new_path.push(file.file_name().context("Direntry has no filename")?);

                        fs::rename(file, &new_path)?;
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
        BotActions::ShowQr => match db::get_config(&mut *bot.get_db_connection().await?).await {
            Ok(config) => {
                println!("Genisis invite qr:");
                qr2term::print_qr(config.genesis_qr)?;
                println!("Bot invite qr:");
                qr2term::print_qr(config.invite_qr)?;
            }
            Err(_) => println!("Bot not configured yet, start the bot first."),
        },
        BotActions::Start => {
            bot.start().await;
            signal::ctrl_c().await?;
        }
    }
    Ok(())
}
