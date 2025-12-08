use crate::config::Config;
use crate::error::Result;
use crate::models::MangaInfo;
use crate::utils;
use crate::utils::codec::{encode_command_button, encode_command_link};
use crate::utils::escape_md_v2;
use std::format;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;
use tracing::info;

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &Config,
    key: Option<String>,
    typ: Option<String>,
    page: Option<i32>,
) -> Result<()> {
    let key = key.unwrap_or("".to_string());
    let typ = typ.unwrap_or("a".to_string());
    let page = page.unwrap_or(1);

    let url = build_search_url(&config.manga.base_url, &key, &typ, page);
    let mangas = if typ == "t" {
        crate::services::manga::parse_cate(&url, &config.manga.base_url).await?
    } else {
        crate::services::manga::parse_search(&url, &config.manga.base_url).await?
    };
    info!("url:{} manga size:{}", url, mangas.len());

    let mut lines = Vec::with_capacity(mangas.len().max(1));
    lines.push(format!(
        "*{}*   🌏{} 📄{}",
        escape_md_v2(&type_nav(&typ, &key)),
        page,
        mangas.len()
    ));
    for m in mangas.iter() {
        lines.push(format_manga_item(m, &config.bot.bot_name));
    }
    let text = lines.join("\n");

    let mut buttons = Vec::with_capacity(2);
    let key_num = utils::cache::search_key_to_num(&key).await;
    if page > 1 {
        buttons.push(encode_command_button(
            "⬅️上一页",
            "csearch",
            &[key_num.to_string(), typ.clone(), (page - 1).to_string()],
        ));
    }
    buttons.push(encode_command_button(
        "下一页➡️",
        "csearch",
        &[key_num.to_string(), typ.clone(), (page + 1).to_string()],
    ));

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}

fn type_nav(typ: &str, key: &str) -> String {
    match typ.as_ref() {
        "u" => format!("用户:{}", key),
        "t" => format!("标签:{}", key),
        _ => format!("全部:{}", key),
    }
}

fn build_search_url(base_url: &str, key: &str, typ: &str, page: i32) -> String {
    let search_key = percent_encoding::utf8_percent_encode(key, percent_encoding::NON_ALPHANUMERIC);
    match typ {
        "u" => format!(
            "{}/q/index.php?q={}&syn=yes&f=user_nicename&s=create_time_DESC&p={page}",
            base_url.trim_end_matches('/'),
            search_key,
        ),
        "t" => format!(
            "{}/albums-index-page-{page}-tag-{}.html",
            base_url.trim_end_matches('/'),
            search_key,
        ),
        _ => format!(
            "{}/q/index.php?q={}&f=_all&syn=yes&s=create_time_DESC&p={page}",
            base_url.trim_end_matches('/'),
            search_key,
        ),
    }
}

fn format_manga_item(m: &MangaInfo, bot_name: &str) -> String {
    let title = escape_md_v2(&m.title);
    let cover_url = &m.cover;
    let total = m.total.max(0);
    let date = escape_md_v2(&m.published);
    let info_url = encode_command_link(bot_name, "info", &[m.id]);
    format!("* [{}]({}) / 📄{} / 📢{} / 👉[{}]({}) ", title, cover_url, total, date, m.id, info_url)
}
