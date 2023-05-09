pub mod bot;
pub mod cli;
pub mod db;
pub mod utils;

use bot::Bot;
use tokio::signal;


#[tokio::main]
async fn main() {
    env_logger::init();
    let mut bot = Bot::new().await;
    bot.start().await;
    signal::ctrl_c().await.unwrap();
}
