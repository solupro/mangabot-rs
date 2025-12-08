use crate::config::Config;
use crate::error::Result;
use crate::models::MangaInfo;
use std::format;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};
use crate::utils;
use crate::utils::codec::{encode_command_button, encode_command_link};
use crate::utils::escape_md_v2;

#[derive(Debug, Clone, Copy)]
enum RankType {
    Day,
    Week,
    Month,
}

impl RankType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "day" | "d" | "1" => Some(Self::Day),
            "week" | "w" | "2" => Some(Self::Week),
            "month" | "m" | "3" => Some(Self::Month),
            _ => Some(Self::Day),
        }
    }
}

fn build_ranking_url(base_url: &str, rank_type: RankType, page: i32) -> String {
    format!(
        "{}/albums-favorite_ranking-page-{}-type-{}.html",
        base_url.trim_end_matches('/'), // 防止双斜杠
        page,
        rank_type.as_str()
    )
}


async fn format_manga_item(m: &MangaInfo, bot_name: &str) -> String {
    let title = escape_md_v2(&m.title);
    let cover_url = &m.cover;
    let rank = m.rank.max(0);
    let total = m.total.max(0);
    let info_link = encode_command_link(bot_name, "info", &[m.id]);

    let author = escape_md_v2(&m.author);
    let author_key_num = utils::cache::search_key_to_num(&author).await;
    let author_link = encode_command_link(bot_name, "csearch", &[author_key_num.to_string(), "u".to_string(), 1.to_string()]);

    format!(
        "*\\#{}* [{}]({}) / 📄{} / ⭐{} / 👤[{}]({}) / 👉[{}]({}) ",
        rank,
        title,
        cover_url,
        total,
        m.fav.max(0),
        author,
        author_link,
        m.id,
        info_link
    )
}

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &Config,
    period: Option<String>,
    page: Option<i32>,
) -> Result<()> {
    let period = period.unwrap_or_else(|| "day".to_string());
    let page = page.unwrap_or(1).clamp(1, 1000);

    let rank_type = RankType::from_str(period.as_str()).unwrap_or(RankType::Day);
    let url = build_ranking_url(&config.manga.base_url, rank_type, page);

    let mangas = crate::services::manga::parse_rank(&url, &config.manga.base_url).await?;

    let mut lines = Vec::with_capacity(mangas.len().max(1));
    lines.push(format!(
            "*排行榜* \\(`{}`\\) 🌏{} 📄{}",
        escape_md_v2(rank_type.as_str()),
        page,
        mangas.len()
    ));
    for m in mangas.iter().take(20) {
        lines.push(format_manga_item(m, &config.bot.bot_name).await);
    }
    let text = lines.join("\n");

    let mut buttons = Vec::with_capacity(2);
    if page > 1 {
        buttons.push(encode_command_button("⬅️上一页", "rank", &[period.clone(), (page - 1).to_string()]));
    }
    buttons.push(encode_command_button("下一页➡️", "rank", &[period.clone(), (page + 1).to_string()]));

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}
