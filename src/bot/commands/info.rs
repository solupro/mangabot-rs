use crate::config::Config;
use crate::error::Result;
use crate::models::MangaDetail;
use crate::utils::codec::{encode_command_button, encode_command_link};
use crate::utils::escape_md_v2;
use crate::{services, utils};
use std::format;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn handle(bot: &Bot, msg: &Message, config: &Config, aid: String) -> Result<()> {
    let info_url = build_info_url(&config.manga.base_url, &aid);
    let manga_detail =
        services::manga::parse_detail(aid.parse::<i64>().unwrap(), &info_url).await?;
    let detail_msg = build_detail_msg(manga_detail, &config.bot.bot_name).await;

    let mut buttons = Vec::with_capacity(2);
    buttons.push(encode_command_button("🏞️预览", "preview", &[aid.clone()]));
    buttons.push(encode_command_button("⏬下载️", "zip", &[aid]));

    bot.send_message(msg.chat.id, detail_msg)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}

async fn build_detail_msg(m: MangaDetail, bot_name: &str) -> String {
    let title = escape_md_v2(&m.title);
    let author = escape_md_v2(&m.author);
    let author_key_num = utils::cache::search_key_to_num(&author).await;
    let author_link = encode_command_link(
        bot_name,
        "csearch",
        &[author_key_num.to_string(), "u".to_string(), 1.to_string()],
    );

    let category = escape_md_v2(&m.category);
    let desc = escape_md_v2(&m.description);
    let cover_url = &m.cover;

    let tags = futures::future::join_all(m.tags.iter().map(|t| {
        let tag = escape_md_v2(t);
        async move {
            let tag_key_num = utils::cache::search_key_to_num(&tag).await;
            let tag_link = encode_command_link(
                bot_name,
                "csearch",
                &[tag_key_num.to_string(), "t".to_string(), 1.to_string()],
            );
            format!("[\\#{tag}]({tag_link})")
        }
    }))
    .await
    .join(" ");

    format!(
        "*[{title}]({cover_url})*\n\n\
         👤 *Author:* [{author}]({author_link})\n\
         📚 *Category:* `{category}`\n\
         🏷 *Tags:* {tags}\n\
         📄 *Size:* `{}`\n\n\
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
