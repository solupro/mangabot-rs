use std::sync::Arc;
use crate::{error::Result, services, telemetry::CommandMetrics};
use teloxide::prelude::*;
use tracing::{info, instrument};
use crate::bot::commands::{start, copy, rank, info, Command};
use crate::utils;

#[instrument(skip(bot, config))]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    config: Arc<crate::config::Config>,
) -> Result<()> {
    // 记录命令指标
    CommandMetrics::record(&cmd);

    match cmd {
        Command::Start(payload) => {
            if let Some(p) = payload {
                if let Ok(cmd2) = utils::codec::decode_command(&p) {
                    return match cmd2 {
                        Command::Rank(period, page) => rank::handle(bot, msg, config, period, page).await,
                        Command::Info(aid) => info::handle(bot, msg, config, aid).await,
                        _ => start::handle(bot, msg).await,
                    };
                }
            }
            start::handle(bot, msg).await
        }
        Command::Copy(say) => copy::handle(bot, msg, say).await,
        Command::Rank(period, page) => rank::handle(bot, msg, config, period, page).await,
        Command::Info(aid) => info::handle(bot, msg, config, aid).await,
    }
}

#[instrument(skip(bot, config))]
pub async fn handle_callback(
    bot: Bot,
    cq: teloxide::types::CallbackQuery,
    config: Arc<crate::config::Config>,
) -> Result<()> {
    if let Some(data) = cq.data.clone() {
        if let Ok(cmd) = utils::codec::decode_command(&data) {
            CommandMetrics::record(&cmd);
            if let Some(msg) = cq.regular_message() {
                match cmd {
                    Command::Rank(period, page) => rank::handle(bot.clone(), msg.clone(), config.clone(), period, page).await?,
                    Command::Info(aid) => info::handle(bot.clone(), msg.clone(), config.clone(), aid).await?,
                    _ => {}
                }
            }
        }
        bot.answer_callback_query(cq.id)
            .text("⏳ 加载中...")
            .show_alert(false) // 顶部小提示（非弹窗）
            .await?;
    }
    Ok(())
}
