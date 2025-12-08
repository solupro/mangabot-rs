use crate::error::Result;
use crate::services;
use crate::utils::codec::encode_command_button;
use std::cmp::min;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, InputFile, InputMedia, InputMediaPhoto};

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &crate::config::Config,
    aid: Option<String>,
    page: Option<i32>,
) -> Result<()> {
    let paid = aid.unwrap_or_default();

    let images_url = crate::bot::commands::build_images_url(&config.manga.base_url, paid.as_str());
    let images =
        services::manga::extract_image_urls(paid.as_str(), &images_url, &config.manga.base_url)
            .await?;

    let limit = usize::try_from(config.manga.preview_size).unwrap_or(5);
    let offset = (page.map(|p| p as usize).unwrap_or(1) - 1) * limit;
    let total = images.len();
    let next = min(offset + limit, total);

    let images = &images[offset..next];

    let media: Vec<_> = images
        .into_iter()
        .map(|url| {
            InputMedia::Photo(InputMediaPhoto {
                media: InputFile::url(url.parse().unwrap()), // 关键：用 url()
                caption: None,
                parse_mode: None,
                caption_entities: None,
                has_spoiler: false,
            })
        })
        .collect();

    bot.send_media_group(msg.chat.id, media).await?;

    if next < total {
        let buttons = vec![encode_command_button(
            "下一页➡️",
            "preview",
            &[paid, (page.unwrap() + 1).to_string()],
        )];
        bot.send_message(msg.chat.id, "🌏浏览更多图片")
            .reply_markup(InlineKeyboardMarkup::new([buttons]))
            .await?;
    }

    Ok(())
}
