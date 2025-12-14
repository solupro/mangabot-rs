use crate::error::Result;
use crate::utils::codec::encode_command_button;
use strum::IntoEnumIterator;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

#[derive(Debug, Clone, Copy)]
pub enum MenuType {
    Rank,
    CateTrz,
    CateDxb,
    CateDp,
    CateHm,
}

impl MenuType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Rank => "排行榜",
            Self::CateTrz => "同人志",
            Self::CateDxb => "单行本",
            Self::CateDp => "短篇",
            Self::CateHm => "韩漫",
        }
    }
    fn as_callback(&self) -> Vec<(String, String, Vec<String>)> {
        match self {
            Self::Rank => super::rank::RankType::iter()
                .map(|t| {
                    (
                        t.as_name().to_string(),
                        "rank".to_string(),
                        vec![t.as_str().to_string()],
                    )
                })
                .collect(),
            Self::CateTrz => super::cate::DoujinshiSub::iter()
                .map(|t| {
                    (
                        t.as_name().to_string(),
                        "cate".to_string(),
                        vec!["trz".to_string(), t.as_str().to_string()],
                    )
                })
                .collect(),
            Self::CateDxb => super::cate::TankoubonSub::iter()
                .map(|t| {
                    (
                        t.as_name().to_string(),
                        "cate".to_string(),
                        vec!["dxb".to_string(), t.as_str().to_string()],
                    )
                })
                .collect(),
            Self::CateDp => super::cate::WebtoonSub::iter()
                .map(|t| {
                    (
                        t.as_name().to_string(),
                        "cate".to_string(),
                        vec!["dp".to_string(), t.as_str().to_string()],
                    )
                })
                .collect(),
            Self::CateHm => super::cate::ShortSub::iter()
                .map(|t| {
                    (
                        t.as_name().to_string(),
                        "cate".to_string(),
                        vec!["hm".to_string(), t.as_str().to_string()],
                    )
                })
                .collect(),
        }
    }
}
pub async fn handle(bot: &Bot, msg: &Message, menu_type: MenuType) -> Result<()> {
    let callbacks = menu_type.as_callback();

    let mut buttons = Vec::with_capacity(callbacks.len());
    for (text, command, args) in callbacks {
        buttons.push(encode_command_button(&text, &command, args.as_slice()));
    }

    bot.send_message(msg.chat.id, menu_type.as_str()).reply_markup(InlineKeyboardMarkup::new([buttons])).await?;

    Ok(())
}
