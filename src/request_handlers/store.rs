use super::WebxdcStatusUpdatePayload;
use crate::{
    bot::State,
    db,
    utils::{send_newest_updates, send_store_xdc, send_update_payload_only},
};
use anyhow::Context as _;
use base64::encode;
use deltachat::{
    chat::{self, ChatId},
    constants,
    context::Context,
    message::MsgId,
};
use log::{info, warn};
use std::sync::Arc;

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        send_store_xdc(context, &state, chat_id, true).await?;
    }
    Ok(())
}

pub async fn handle_status_update(
    context: &Context,
    state: Arc<State>,
    msg_id: MsgId,
    payload: WebxdcStatusUpdatePayload,
) -> anyhow::Result<()> {
    match payload {
        WebxdcStatusUpdatePayload::UpdateRequest { serial, apps } => {
            info!("Handling store update request");

            // Get all updating xdcs
            let mut updating = vec![];
            let conn = &mut *state.db.acquire().await?;
            for (app_id, ref tag_name) in apps {
                if db::maybe_get_greater_tag_name(conn, &app_id, tag_name).await? {
                    updating.push(app_id);
                }
            }

            info!("Updating multiple client apps: {:?}", updating);

            send_newest_updates(
                context,
                msg_id,
                &mut *state.db.acquire().await?,
                serial,
                updating.clone(),
            )
            .await?;

            // Send updates
            for app_id in &updating {
                let context = context.clone();
                let state = state.clone();
                let app_id = app_id.clone();
                let resp = handle_download(&state, app_id).await;
                send_update_payload_only(&context, msg_id, resp).await?;
            }
        }
        WebxdcStatusUpdatePayload::Download { app_id } => {
            info!("Handling store download");
            let resp = handle_download(&state, app_id).await;
            send_update_payload_only(context, msg_id, resp).await?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_download(state: &State, app_id: String) -> WebxdcStatusUpdatePayload {
    match get_webxdc_data(state, &app_id).await {
        Ok((data, name)) => WebxdcStatusUpdatePayload::DownloadOkay { data, name, app_id },
        Err(e) => {
            warn!("Error while handling download request: {}", e);
            WebxdcStatusUpdatePayload::DownloadError {
                error: e.to_string(),
                app_id,
            }
        }
    }
}

/// Handles a request to download a store app.
/// Returns the base64 encoded webxdc and the name of the app.
async fn get_webxdc_data(state: &State, app_id: &str) -> anyhow::Result<(String, String)> {
    let app = db::get_app_info_for_app_id(&mut *state.db.acquire().await?, app_id).await?;
    Ok((
        encode(
            &tokio::fs::read(
                app.xdc_blob_path
                    .to_str()
                    .context("Can't covert file '{file:?}' to str")?,
            )
            .await?,
        ),
        app.name,
    ))
}
