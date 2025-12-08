use crate::config::Config;
use crate::error::Result;
use crate::models::MangaInfo;
use crate::utils::codec::{encode_command_button, encode_command_link};
use crate::utils::escape_md_v2;
use std::format;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

#[derive(Debug, Clone, Copy)]
enum Category {
    DOUJINSHI(DoujinshiSub), // 同人志
    TANKOUBON(TankoubonSub), // 单行本
    SHORT(ShortSub),         // 短篇
    WEBTOON(WebtoonSub),     // 韩漫
}

#[derive(Debug, Clone, Copy)]
enum DoujinshiSub {
    ALL, // 全部
    ZH,  // 汉化
    JA,  // 日语
    EN,  // 英语
    CG,  // CG
    COSPLAY,
    _3D, // 3D
    AI,  // AI
}

#[derive(Debug, Clone, Copy)]
enum TankoubonSub {
    ALL, // 全部
    ZH,  // 汉化
    JA,  // 日语
    EN,  // 英语
}

#[derive(Debug, Clone, Copy)]
enum ShortSub {
    ALL, // 全部
    ZH,  // 汉化
    JA,  // 日语
    EN,  // 英语
}

#[derive(Debug, Clone, Copy)]
enum WebtoonSub {
    ALL, // 全部
    ZH,  // 汉化
    SRC, // 生肉
}

impl Category {
    fn from_str(cate: &str, sub: &str) -> Self {
        match cate.to_ascii_lowercase().as_str() {
            "同人志" | "doujinshi" | "trz" => match sub.to_ascii_lowercase().as_str() {
                "全部" | "all" | "qb" => Self::DOUJINSHI(DoujinshiSub::ALL),
                "汉化" | "zh" | "hh" => Self::DOUJINSHI(DoujinshiSub::ZH),
                "日语" | "ja" | "ry" => Self::DOUJINSHI(DoujinshiSub::JA),
                "英语" | "en" | "yy" => Self::DOUJINSHI(DoujinshiSub::EN),
                "CG" | "cg" => Self::DOUJINSHI(DoujinshiSub::CG),
                "COSPLAY" | "cosplay" | "cos" => Self::DOUJINSHI(DoujinshiSub::COSPLAY),
                "3D" | "3d" => Self::DOUJINSHI(DoujinshiSub::_3D),
                "AI" | "ai" => Self::DOUJINSHI(DoujinshiSub::AI),
                _ => Self::DOUJINSHI(DoujinshiSub::ZH),
            },
            "单行本" | "tankoubon" | "dxb" => match sub.to_ascii_lowercase().as_str() {
                "全部" | "all" | "qb" => Self::TANKOUBON(TankoubonSub::ALL),
                "汉化" | "zh" | "hh" => Self::TANKOUBON(TankoubonSub::ZH),
                "日语" | "ja" | "ry" => Self::TANKOUBON(TankoubonSub::JA),
                "英语" | "en" | "yy" => Self::TANKOUBON(TankoubonSub::EN),
                _ => Self::TANKOUBON(TankoubonSub::ZH),
            },
            "短篇" | "short" | "sc" | "dp" => match sub.to_ascii_lowercase().as_str() {
                "全部" | "all" | "qb" => Self::SHORT(ShortSub::ALL),
                "汉化" | "zh" | "hh" => Self::SHORT(ShortSub::ZH),
                "日语" | "ja" | "ry" => Self::SHORT(ShortSub::JA),
                "英语" | "en" | "yy" => Self::SHORT(ShortSub::EN),
                _ => Self::SHORT(ShortSub::ZH),
            },
            "韩漫" | "webtoon" | "kt" | "hm" => match sub.to_ascii_lowercase().as_str() {
                "全部" | "all" | "qb" => Self::WEBTOON(WebtoonSub::ALL),
                "汉化" | "zh" | "hh" => Self::WEBTOON(WebtoonSub::ZH),
                "生肉" | "src" => Self::WEBTOON(WebtoonSub::SRC),
                _ => Self::WEBTOON(WebtoonSub::ZH),
            },
            _ => Self::DOUJINSHI(DoujinshiSub::ZH),
        }
    }

    fn to_cate_info(&self) -> (String, &str) {
        match self {
            Self::DOUJINSHI(sub) => {
                let cate_name = "同人志";
                let (sub_name, cate_type) = match sub {
                    DoujinshiSub::ALL => ("全部", "5"),
                    DoujinshiSub::ZH => ("汉化", "1"),
                    DoujinshiSub::JA => ("日语", "12"),
                    DoujinshiSub::EN => ("英语", "16"),
                    DoujinshiSub::CG => ("CG", "2"),
                    DoujinshiSub::COSPLAY => ("COSPLAY", "3"),
                    DoujinshiSub::_3D => ("3D", "22"),
                    DoujinshiSub::AI => ("AI", "37"),
                };
                (format!("{}-{}", cate_name, sub_name), cate_type)
            }
            Self::TANKOUBON(sub) => {
                let cate_name = "单行本";
                let (sub_name, cate_type) = match sub {
                    TankoubonSub::ALL => ("全部", "6"),
                    TankoubonSub::ZH => ("汉化", "9"),
                    TankoubonSub::JA => ("日语", "13"),
                    TankoubonSub::EN => ("英语", "17"),
                };
                (format!("{}-{}", cate_name, sub_name), cate_type)
            }
            Self::SHORT(sub) => {
                let cate_name = "短篇";
                let (sub_name, cate_type) = match sub {
                    ShortSub::ALL => ("全部", "7"),
                    ShortSub::ZH => ("汉化", "10"),
                    ShortSub::JA => ("日语", "14"),
                    ShortSub::EN => ("英语", "18"),
                };
                (format!("{}-{}", cate_name, sub_name), cate_type)
            }
            Self::WEBTOON(sub) => {
                let cate_name = "韩漫";
                let (sub_name, cate_type) = match sub {
                    WebtoonSub::ALL => ("全部", "19"),
                    WebtoonSub::ZH => ("汉化", "20"),
                    WebtoonSub::SRC => ("生肉", "12"),
                };
                (format!("{}-{}", cate_name, sub_name), cate_type)
            }
        }
    }
}

fn build_cate_url(base_url: &str, cate_num: &str, page: i32) -> String {
    format!(
        "{}/albums-index-page-{}-cate-{}.html",
        base_url.trim_end_matches('/'), // 防止双斜杠
        page,
        cate_num
    )
}

fn format_manga_item(m: &MangaInfo, bot_name: &str) -> String {
    let title = escape_md_v2(&m.title);
    let cover_url = &m.cover;
    let total = m.total.max(0);
    let date = escape_md_v2(&m.published);
    let info_url = encode_command_link(bot_name, "info", &[m.id]);
    format!("* [{}]({}) / 📄{} / 📢{} / 👉[{}]({}) ", title, cover_url, total, date, m.id, info_url)
}

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &Config,
    cate: Option<String>,
    sub: Option<String>,
    page: Option<i32>,
) -> Result<()> {
    let cate = cate.unwrap_or_else(|| "同人志".to_string());
    let sub = sub.unwrap_or_else(|| "汉化".to_string());
    let page = page.unwrap_or(1).clamp(1, 1000);

    let cate_type = Category::from_str(cate.as_str(), sub.as_str());
    let (cate_nav, cate_num) = cate_type.to_cate_info();
    let url = build_cate_url(&config.manga.base_url, cate_num, page);

    let mangas = crate::services::manga::parse_cate(&url, &config.manga.base_url).await?;

    let mut lines = Vec::with_capacity(mangas.len().max(1));
    lines.push(format!("*{}*   🌏{} 📄{}", escape_md_v2(cate_nav.as_str()), page, mangas.len()));
    for m in mangas.iter().take(20) {
        lines.push(format_manga_item(m, &config.bot.bot_name));
    }
    let text = lines.join("\n");

    let mut buttons = Vec::with_capacity(2);
    if page > 1 {
        buttons.push(encode_command_button(
            "⬅️上一页",
            "cate",
            &[cate.clone(), sub.clone(), (page - 1).to_string()],
        ));
    }
    buttons.push(encode_command_button(
        "下一页➡️",
        "cate",
        &[cate.clone(), sub.clone(), (page + 1).to_string()],
    ));

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}
