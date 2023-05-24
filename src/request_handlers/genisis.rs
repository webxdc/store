use std::sync::Arc;

use clap::{CommandFactory, FromArgMatches};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;

use crate::{bot::State, bot_commands::Genesis};

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    msg_id: MsgId,
) -> anyhow::Result<()> {
    let msg = Message::load_from_db(context, msg_id).await?;
    if let Some(text) = msg.get_text() {
        // only react to messages with right keywoard
        if let Some(text) = text.strip_prefix('/') {
            info!("Handling command to bot");
            match <Genesis as CommandFactory>::command().try_get_matches_from(text.split(' ')) {
                Ok(mut matches) => {
                    let res = <Genesis as FromArgMatches>::from_arg_matches_mut(&mut matches)?;

                    match res.join {
                        crate::bot_commands::GroupName::Join { name } => {
                            let contact_id = msg.get_from_id();

                            let chat_id = match name {
                                crate::bot_commands::BotGroup::Genesis => {
                                    state.config.genesis_group
                                }
                                crate::bot_commands::BotGroup::Reviewee => {
                                    state.config.reviewee_group
                                }
                                crate::bot_commands::BotGroup::Tester => state.config.tester_group,
                            };

                            chat::add_contact_to_chat(context, chat_id, contact_id).await?
                        }
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
