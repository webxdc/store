pub mod bot;
pub mod db;
pub mod cli;
pub mod server;
pub mod shared;
pub mod utils;

use bot::Bot;
use tokio::signal;

pub const PORT: usize = 65005;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut bot = Bot::new().await;
    bot.start().await;
    signal::ctrl_c().await.unwrap();
    bot.stop().await;
}
