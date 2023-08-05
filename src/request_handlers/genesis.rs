use anyhow::Result;
use deltachat::{chat::ChatId, context::Context, message::MsgId};
use std::sync::Arc;

use crate::bot::State;

pub async fn handle_message(
    _context: &Context,
    _state: Arc<State>,
    _chat_id: ChatId,
    _msg_id: MsgId,
) -> Result<()> {
    Ok(())
}
