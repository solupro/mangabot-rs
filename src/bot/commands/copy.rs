use teloxide::prelude::*;
use crate::error::Result;
use std::format;

pub async fn handle(bot: Bot, msg: Message, say: String) -> Result<()> {
    let copy_msg = format!(
        "👋 *{}* say: {}",
        msg.from.as_ref().map(|u| u.first_name.clone()).unwrap_or_else(|| "Unknown user".to_string()),
        say
    );

    bot.send_message(msg.chat.id, copy_msg)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}
