use std::sync::Arc;

use clap::{CommandFactory, FromArgMatches};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;

use crate::{bot::State, bot_commands::Genesis, db};

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    msg_id: MsgId,
) -> anyhow::Result<()> {
    let msg = Message::load_from_db(context, msg_id).await?;
    let conn = &mut *state.db.acquire().await?;

    if let Some(text) = msg.get_text() {
        // only react to messages commands
        if let Some(text) = text.strip_prefix('/') {
            info!("Handling command to bot");
            match <Genesis as CommandFactory>::command().try_get_matches_from(text.split(' ')) {
                Ok(mut matches) => {
                    let res = <Genesis as FromArgMatches>::from_arg_matches_mut(&mut matches)?;
                    let contact_id = msg.get_from_id();

                    if let Some(group) = res.join {
                        info!("Adding user {contact_id} to group {group:?}");
                        let chat_id = match group {
                            crate::bot_commands::BotGroup::Publisher => {
                                db::add_publisher(conn, contact_id).await.ok();
                                state.config.reviewee_group
                            }
                            crate::bot_commands::BotGroup::Tester => {
                                db::add_tester(conn, contact_id).await.ok();
                                state.config.tester_group
                            }
                        };
                        let chat = chat::Chat::load_from_db(context, chat_id).await?;
                        if !chat.is_promoted() {
                            chat::send_text_msg(
                                context,
                                chat_id,
                                "Welcome to the new group".into(),
                            )
                            .await?;
                        }
                        chat::add_contact_to_chat(context, chat_id, contact_id).await?;
                    }
                }
                Err(e) => {
                    chat::send_text_msg(context, chat_id, format!("{e}")).await?;
                }
            };
        }
    }
    Ok(())
}
