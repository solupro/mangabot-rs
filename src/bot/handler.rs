use std::sync::Arc;
use crate::{error::Result, services, telemetry::CommandMetrics};
use teloxide::prelude::*;
use tracing::{info, instrument};
use crate::bot::commands::{start, copy, Command};
use crate::config::Config;

// 2024: 改进的错误传播
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
        Command::Start => start::handle(bot, msg).await,
        Command::Copy(say) => copy::handle(bot, msg, say).await,
    }
}
