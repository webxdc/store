//! Handlers for the different messages the bot receives
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub struct Chat {
    pub chat_type: ChatType,
}

pub enum ChatType {
    Release,
    Review,
    Reviewee,
    Testers,
    Shop,
}

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub author_name: String,
    pub author_email: String,
    pub source_code_url: String,
    pub image: String,
    pub description: String,
    pub xdc_blob_url: String,
    pub version: String,
}

pub mod review {}

pub mod shop {
    use std::sync::Arc;

    use deltachat::{
        chat::{self, ChatId},
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use log::info;
    use serde::Deserialize;
    use serde_json::json;
    use ts_rs::TS;

    #[derive(TS, Deserialize)]
    #[ts(export)]
    enum RequestType {
        Update,
    }

    #[derive(TS, Deserialize)]
    #[ts(export)]
    struct WebxdcRequest {
        request_type: RequestType,
    }

    use crate::bot::State;

    pub async fn handle_message(context: &Context, chat_id: ChatId) -> anyhow::Result<()> {
        // Handle normal messages to the bot (resend the store itself).
        chat::send_text_msg(
            context,
            chat_id,
            r#"
            Welcome to the appstore bot! 
            I will shortly send you the appstore itself wher you can explore new apps."#
                .to_string(),
        )
        .await?;

        let mut webxdc_msg = Message::new(Viewtype::Webxdc);
        webxdc_msg.set_file("webxdc.xdc", None);
        chat::send_msg(context, chat_id, &mut webxdc_msg).await?;

        Ok(())
    }

    pub async fn handle_status_update(
        context: &Context,
        state: Arc<State>,
        _chat_id: ChatId,
        msg_id: MsgId,
        update: String,
    ) -> anyhow::Result<()> {
        if let Ok(req) = serde_json::from_str::<WebxdcRequest>(&update) {
            match req.request_type {
                RequestType::Update => {
                    context
                        .send_webxdc_status_update_struct(
                            msg_id,
                            deltachat::webxdc::StatusUpdateItem {
                                payload: json! {state.get_apps()},
                                ..Default::default()
                            },
                            "",
                        )
                        .await?;
                    info!("Handling store update")
                }
            }
        } else {
            info!("Can't handle update")
        }
        Ok(())
    }
}
