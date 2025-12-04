use crate::bot::commands::{Command, copy, info, preview, rank, start, zip};
use crate::utils;
use crate::{error::Result, services, telemetry::CommandMetrics};
use std::sync::Arc;
use teloxide::prelude::*;
use tracing::{info, instrument};

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
                    return handle_codec(bot, &msg, cmd2, config).await;
                }
            }
            start::handle(bot, msg).await
        }
        Command::Copy(say) => copy::handle(bot, msg, say).await,
        Command::Rank(period, page) => rank::handle(bot, msg, config, period, page).await,
        Command::Info(aid) => info::handle(bot, msg, config, aid).await,
        Command::Preview(aid, page) => preview::handle(bot, msg, config, aid, page).await,
        Command::Zip(aid) => zip::handle(bot, msg, config, aid).await,
    }
}

#[instrument(skip(bot, config))]
pub async fn handle_callback(
    bot: Bot,
    cq: CallbackQuery,
    config: Arc<crate::config::Config>,
) -> Result<()> {
    if let Some(data) = cq.data.clone() {
        let mut delete_flag = false;
        let mut clone_msg: Option<Message> = None;
        if let Ok(cmd) = utils::codec::decode_command(&data) {
            CommandMetrics::record(&cmd);

            delete_flag = match cmd {
                Command::Preview(_, _) => true,
                _ => false,
            };

            if let Some(msg) = cq.regular_message() {
                clone_msg = Some(msg.clone());
                handle_codec(bot.clone(), msg, cmd, config).await?;
            }
        }
        bot.answer_callback_query(cq.id)
            .text("⏳ 加载中...")
            .show_alert(false) // 顶部小提示（非弹窗）
            .await?;

        if delete_flag && clone_msg.is_some() {
            bot.delete_message(clone_msg.as_ref().unwrap().chat.id, clone_msg.as_ref().unwrap().id).await?;
        }
    }
    Ok(())
}

async fn handle_codec(
    bot: Bot,
    msg: &Message,
    cmd: Command,
    config: Arc<crate::config::Config>,
) -> Result<()> {

    match cmd {
        Command::Rank(period, page) => rank::handle(bot, msg.clone(), config, period, page).await?,
        Command::Info(aid) => info::handle(bot, msg.clone(), config, aid).await?,
        Command::Preview(aid, page) => preview::handle(bot, msg.clone(), config, aid, page).await?,
        Command::Zip(aid) => zip::handle(bot, msg.clone(), config, aid).await?,
        _ => {}
    }

    Ok(())
}
