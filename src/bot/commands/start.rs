use teloxide::prelude::*;
use crate::error::Result;
use std::format;

pub async fn handle(bot: Bot, msg: Message) -> Result<()> {
    let welcome_msg = format!(
        "👋 你好 *{}*！\n\n欢迎使用本机器人\n\n输入 /help 查看可用命令",
        msg.from.map(|u| u.first_name).unwrap_or("Unknown user".to_string())
    );

    bot.send_message(msg.chat.id, welcome_msg)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}