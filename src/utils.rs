//! Utility functions

use anyhow::{bail, Context as _, Result};
use deltachat::{
    chat::{self, ChatId},
    config::Config,
    context::Context,
    message::{MsgId, Viewtype},
};
use std::env;

pub async fn configure_from_env(ctx: &Context) -> Result<()> {
    let addr = env::var("addr")?;
    ctx.set_config(Config::Addr, Some(&addr)).await?;
    let pw = env::var("mail_pw")?;
    ctx.set_config(Config::MailPw, Some(&pw)).await?;
    //ctx.set_config(Config::Bot, Some("1")).await?;
    ctx.set_config(Config::E2eeEnabled, Some("1")).await?;
    ctx.configure()
        .await
        .context("configure failed, you might have wrong credentials")?;
    Ok(())
}

async fn get_appstore_xdc(context: &Context, chat_id: ChatId) -> anyhow::Result<MsgId> {
    let mut msg_ids = chat::get_chat_media(
        context,
        Some(chat_id),
        Viewtype::Webxdc,
        Viewtype::Unknown,
        Viewtype::Unknown,
    )
    .await?;
    if let Some(msg_id) = msg_ids.pop() {
        Ok(msg_id)
    } else {
        bail!("no appstore xdc in chat");
    }
}
