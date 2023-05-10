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

pub mod review {}

pub mod shop {
    use std::sync::Arc;

    use deltachat::{
        chat::{self, ChatId},
        context::Context,
        message::{Message, MsgId, Viewtype},
    };

    use crate::bot::State;

    pub async fn handle_message(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> anyhow::Result<()> {
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
        chat_id: ChatId,
        msg_id: MsgId,
        update: String,
    ) -> anyhow::Result<()> {
        println!("{update}");
        Ok(())
    }
}
