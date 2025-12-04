use teloxide::prelude::*;
use crate::error::Result;
use std::format;
use std::sync::Arc;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::config::Config;
use crate::models::MangaDetail;
use crate::services;
use crate::utils::codec::encode_command;
use crate::utils::escape_md_v2;

pub async fn handle(bot: Bot, msg: Message, config: Arc<Config>, aid: String) -> Result<()> {

    let info_url = build_info_url(&config.base_url, &aid);
    let manga_detail = services::manga::parse_detail(aid.parse::<i64>().unwrap(), &info_url).await?;
    let detail_msg = build_detail_msg(manga_detail);

    let mut buttons = Vec::with_capacity(2);
    let preview_data = encode_command("preview", &[aid.clone()]).unwrap();
    buttons.push(InlineKeyboardButton::callback("预览", preview_data));

    let zip_data = encode_command("zip", &[aid]).unwrap();
    buttons.push(InlineKeyboardButton::callback("下载️", zip_data));

    bot.send_message(msg.chat.id, detail_msg)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}

fn build_detail_msg(m: MangaDetail) -> String {
    let title = escape_md_v2(&m.title);
    let author = escape_md_v2(&m.author);
    let category = escape_md_v2(&m.category);
    let desc = escape_md_v2(&m.description);
    let cover_url = &m.cover;

    let tags = m
        .tags
        .iter()
        .map(|t| format!("\\#{}", escape_md_v2(t)))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        "*[{title}]({cover_url})*\n\n\
         👤 *Author:* `{author}`\n\
         📚 *Category:* `{category}`\n\
         🏷 *Tags:* {tags}\n\
         📄 *Pages:* `{}`\n\n\
         {desc}",
        m.total
    )
}

fn build_info_url(base_url: &str, aid: &str) -> String {
    format!(
        "{}/photos-index-aid-{}.html",
        base_url.trim_end_matches('/'), // 防止双斜杠
        aid
    )
}
