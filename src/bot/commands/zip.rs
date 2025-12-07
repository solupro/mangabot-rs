use crate::error::{BotError, Result};
use crate::{services, utils};
use std::format;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageId};
use tracing::error;

static DOC_LIMIT_SIZE: u64 = 50 * 1024 * 1024;

pub async fn handle(
    bot: &Bot,
    msg: &Message,
    config: &crate::config::Config,
    aid: i64,
) -> Result<()> {
    if 0 == aid {
        return Err(BotError::ParseError(
            "aid is required or parse error".to_string(),
        ));
    }
    let sid = aid.to_string();

    let info_url = super::info::build_info_url(&config.manga.base_url, &sid);
    let info = services::manga::parse_detail(aid, &info_url).await?;
    let images_url = crate::bot::commands::build_images_url(&config.manga.base_url, &sid);
    let images =
        services::manga::extract_image_urls(&sid, &images_url, &config.manga.base_url).await?;
    if images.is_empty() {
        return Err(BotError::ParseError(format!(
            "no images found for aid {}",
            aid
        )));
    }

    let reply_msg = bot
        .send_message(
            msg.chat.id,
            format!(
                "【{}】\n\n {}",
                utils::escape_md_v2(&info.title),
                utils::escape_md_v2("⬇️后台下载中，稍后推送...")
            ),
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;


    let bot_clone = bot.clone();
    let chat_id = msg.chat.id;
    let reply_msg_id = reply_msg.id;
    let title = info.title.clone();
    let download_path = config.server.download_path.clone();
    let concurrency = config.server.download_concurrency;
    let images_owned = images; // 转移所有权

    tokio::spawn(async move {
        let result = download_task(
            bot_clone.clone(),
            chat_id,
            reply_msg_id,
            title,
            download_path,
            concurrency,
            images_owned,
        )
        .await;

        if let Err(e) = result {
            error!("后台下载任务失败: {:?}", e);
            // 发送错误消息
            let _ = bot_clone.send_message(chat_id, format!("下载失败: {:?}", e))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await;
        }
    });

    Ok(())
}

async fn download_task(
    bot: Bot,
    chat_id: ChatId,
    reply_msg_id: MessageId,
    title: String,
    download_path: String,
    concurrency: usize,
    images: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manga_dir = format!("{}/{}", download_path, title);
    if tokio::fs::metadata(&manga_dir).await.is_err() {
        tokio::fs::create_dir_all(&manga_dir)
            .await
            .map_err(|e| format!("创建目录失败 {}: {}", manga_dir, e))?;
    }

    utils::http::download_batch(images, &manga_dir, concurrency).await;

    let zip_path = format!("{}/{}.zip", download_path, title);
    utils::zip::compress_dir(&manga_dir, &zip_path)
        .map_err(|e| format!("压缩失败: {:?}", e))?;

    if let Ok(zip_meta) = std::fs::metadata(&zip_path) {
        if zip_meta.len() < DOC_LIMIT_SIZE {
            bot.send_document(chat_id, InputFile::file(&zip_path)).await?;
        } else {
            bot.send_message(chat_id, "文件过大，返回下载token（待实现）")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    }

    // 删除临时提示消息
    bot.delete_message(chat_id, reply_msg_id).await?;

    Ok(())
}