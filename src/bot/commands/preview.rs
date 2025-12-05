use std::cmp::min;
use crate::error::Result;
use crate::services;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, InputMedia, InputMediaPhoto};
use crate::utils::codec::encode_command;

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &crate::config::Config,
    aid: Option<String>,
    page: Option<i32>,
) -> Result<()> {
    let paid = aid.unwrap_or_default();

    let images_url = crate::bot::commands::build_images_url(&config.manga.base_url, paid.as_str());
    let images = services::manga::extract_image_urls(&images_url, &config.manga.base_url).await?;

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
                show_caption_above_media: false,
                has_spoiler: false,
            })
        })
        .collect();

    bot.send_media_group(msg.chat.id, media).await?;

    if next < total {
        let mut buttons = Vec::with_capacity(1);
        let next_data = encode_command("preview", &[paid, (page.unwrap() + 1).to_string()]).unwrap();
        buttons.push(InlineKeyboardButton::callback("下一页➡️", next_data));

        bot.send_message(msg.chat.id, "浏览更多图片")
            .reply_markup(InlineKeyboardMarkup::new([buttons]))
            .await?;
    }

    Ok(())
}
