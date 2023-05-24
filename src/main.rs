mod bot;
mod bot_commands;
mod cli;
mod db;
mod messages;
mod request_handlers;
mod utils;

use bot::Bot;
use clap::Parser;
use cli::{BotActions, BotCli};
use log::info;
use tokio::fs;
use tokio::signal;

use crate::request_handlers::AppInfo;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import => {
            info!("importing webxdcs from '/import/");
            let files = std::fs::read_dir("/import/")
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|e| e.is_file());

            for file in files {
                if file
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(".xdc")
                {
                    let mut app_info = AppInfo::default();
                    app_info.update_from_xdc(file.clone()).await.unwrap();
                    let missing = app_info.generate_missing_list();
                    if missing.len() < 1 {
                        let mut new_path = file.parent().unwrap().to_path_buf();
                        new_path.push("/xdcs");
                        new_path.push(file.file_name().unwrap());
                        fs::rename(file, new_path).await.unwrap()
                    } else {
                        println!(
                            "The app {} is missing some data: {:?}",
                            file.as_os_str().to_str().unwrap(),
                            missing
                        )
                    }
                }
            }
        }
        BotActions::Start => {
            let mut bot = Bot::new().await;
            bot.start().await;
            signal::ctrl_c().await.unwrap();
        }
    }
}
