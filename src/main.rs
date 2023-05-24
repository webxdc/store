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
use surrealdb::sql::Id;
use surrealdb::sql::Thing;
use tokio::fs;
use tokio::signal;

use crate::db::DB;
use crate::request_handlers::AppInfo;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = BotCli::parse();

    match &cli.action {
        BotActions::Import => {
            info!("importing webxdcs from 'import/");
            let db = DB::new("bot.db").await;
            let files: Vec<_> = std::fs::read_dir("import/")
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|e| e.is_file())
                .collect();

            if files.is_empty() {
                println!("no xdcs to add in ./import")
            }

            for file in &files {
                if file
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(".xdc")
                {
                    let mut app_info = AppInfo::default();
                    app_info.update_from_xdc(file.clone()).await.unwrap();
                    app_info.active = true;
                    app_info.author_name = "appstore bot".to_string();
                    app_info.author_email = Some("appstorebot@testrun.org".to_string());
                    app_info.description = "Some description".to_string();

                    let missing = app_info.generate_missing_list();
                    if missing.is_empty() {
                        let mut new_path = file.parent().unwrap().parent().unwrap().to_path_buf();
                        new_path.push("xdcs");
                        new_path.push(file.file_name().unwrap());
                        fs::rename(file, &new_path).await.unwrap();
                        app_info.xdc_blob_dir = Some(new_path);

                        db.create_app_info(
                            &app_info,
                            Thing {
                                tb: "app_info".to_string(),
                                id: Id::rand(),
                            },
                        )
                        .await
                        .unwrap();
                        println!("Added {} to apps", app_info.name);
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
