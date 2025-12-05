use std::sync::Arc;
use crate::bot::commands::{Command, copy, info, preview, rank, start, zip};
use crate::utils;
use crate::{error::Result, telemetry::CommandMetrics};
use teloxide::prelude::*;
use tracing::{warn, debug, instrument};

/// 统一的命令分发核心，返回是否需要删除原消息
#[instrument(skip(bot, config))]
async fn dispatch_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    config: &Arc<crate::config::Config>,
) -> Result<bool> {

    let cmd = resolve_command(cmd);

    CommandMetrics::record(&cmd);
    let should_delete = match cmd {
        Command::Preview(_, Some(page)) => page > 1,
        _ => false,
    };

    match cmd {
        Command::Start(payload) => start::handle(&bot, &msg).await?,
        Command::Copy(say) => copy::handle(&bot, &msg, say).await?,
        Command::Rank(period, page) => rank::handle(&bot, &msg, &config, period, page).await?,
        Command::Info(aid) => info::handle(&bot, &msg, &config, aid).await?,
        Command::Preview(aid, page) => preview::handle(&bot, &msg, &config, aid, page).await?,
        Command::Zip(aid) => zip::handle(&bot, &msg, &config, aid).await?,
    }

    Ok(should_delete)
}

static MAX_DEPTH: usize = 5;
fn resolve_command(mut cmd: Command) -> Command {
    let mut depth = 0;

    while let Command::Start(Some(ref payload)) = cmd {
        if depth >= MAX_DEPTH {
            warn!("Command decode recursion limit reached");
            break;
        }

        match utils::codec::decode_command(payload) {
            Ok(decoded) => {
                cmd = decoded;
                depth += 1;
            }
            Err(_) => {
                // 解码失败，保留当前 Start 命令
                break;
            }
        }
    }

    cmd
}

#[instrument(skip_all)]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    config: Arc<crate::config::Config>,
) -> Result<()> {

    if !config.is_admin(msg.from.as_ref().unwrap().id.0) {
        bot.send_message(msg.chat.id, "❌没权限操作")
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        return Ok(());
    }

    dispatch_command(bot, msg, cmd, &config).await?;
    Ok(())
}

#[instrument(skip_all)]
pub async fn handle_callback(
    bot: Bot,
    cq: CallbackQuery,
    config: Arc<crate::config::Config>,
) -> Result<()> {
    let Some(data) = cq.data.as_deref() else { return Ok(()); };

    if !config.is_admin(cq.from.id.0) {
        bot.answer_callback_query(cq.id.clone())
            .text("❌没权限操作")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let cmd = match utils::codec::decode_command(data) {
        Ok(cmd) => cmd,
        Err(e) => {
            debug!(error = %e, "Failed to decode callback command");
            bot.answer_callback_query(cq.id.clone())
                .text("❌ 无效的操作数据")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    bot.answer_callback_query(cq.id.clone())
        .text("⏳ 处理中...")
        .show_alert(false)
        .await?;

    if let Some(msg) = cq.regular_message() {
        if dispatch_command(bot.clone(), msg.clone(), cmd, &config).await? {
            bot.delete_message(msg.chat.id, msg.id).await?;
        }
    }

    Ok(())
}