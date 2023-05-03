//! Utility functions

use anyhow::{Context as _, Result};
use deltachat::{
    chat::{send_text_msg, ChatId},
    config::Config,
    context::Context,
};
use std::env;

pub async fn configure_from_env(ctx: &Context) -> Result<()> {
    let addr = env::var("addr")?;
    ctx.set_config(Config::Addr, Some(&addr)).await?;
    let pw = env::var("mail_pw")?;
    ctx.set_config(Config::MailPw, Some(&pw)).await?;
    ctx.set_config(Config::Bot, Some("1")).await?;
    ctx.set_config(Config::E2eeEnabled, Some("1")).await?;
    ctx.configure()
        .await
        .context("configure failed, you might have wrong credentials")?;
    Ok(())
}

pub async fn send_text_to_all(chats: &[ChatId], msg: &str, ctx: &Context) -> anyhow::Result<()> {
    for chat_id in chats {
        send_text_msg(ctx, *chat_id, msg.to_string()).await?;
    }
    Ok(())
}
