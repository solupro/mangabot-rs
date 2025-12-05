use crate::config::Config;
use crate::error::Result;
use crate::models::MangaInfo;
use std::format;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use crate::utils::codec::encode_command;
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


fn format_manga_item(m: &MangaInfo, bot_name: &str) -> String {
    let title = escape_md_v2(&m.title);
    let cover_url = &m.cover;
    let rank = m.rank.max(0);
    let total = m.total.max(0);
    let info_act = encode_command("info", &[m.id]).unwrap();
    let info_url = format!("https://t.me/{}?start={}", bot_name, info_act);
    format!(
        "*\\#{}* [{}]({}) — 共{}张 — {}收藏 — id [{}]({}) ",
        rank,
        title,
        cover_url,
        total,
        m.fav.max(0),
        m.id,
        info_url
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

    let mangas = crate::services::manga::parse_rank(&url).await?;

    let mut lines = Vec::with_capacity(mangas.len().max(1));
    lines.push(format!(
        "*排行榜* \\(`{}`\\) 第 {} 页",
        escape_md_v2(rank_type.as_str()),
        page
    ));
    for m in mangas.iter().take(20) {
        lines.push(format_manga_item(m, &config.bot.bot_name));
    }
    let text = lines.join("\n");

    let prev_data = if page > 1 {
        Some(encode_command("rank", &[period.clone(), (page - 1).to_string()]).unwrap())
    } else {
        None
    };
    let next_data = Some(encode_command("rank", &[period.clone(), (page + 1).to_string()]).unwrap());
    let mut buttons = Vec::with_capacity(2);
    if let Some(prev_data) = prev_data {
        buttons.push(InlineKeyboardButton::callback("⬅️上一页", prev_data));
    }
    if let Some(next_data) = next_data {
        buttons.push(InlineKeyboardButton::callback("下一页➡️", next_data));
    }

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}
