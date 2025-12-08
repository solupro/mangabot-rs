use crate::bot::commands::Command;
use std::sync::Arc;
use teloxide::dispatching::HandlerExt;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::Update;
use teloxide::{Bot, dptree};

pub mod commands;
pub mod handler;

pub async fn run(bot: Bot, config: crate::config::Config) -> crate::error::Result<()> {
    let config = Arc::new(config);
    let handler =
        dptree::entry()
            .branch(Update::filter_message().filter_command::<Command>().endpoint(
                |bot: Bot,
                 msg: teloxide::types::Message,
                 cmd: Command,
                 config: Arc<crate::config::Config>| async move {
                    handler::handle_command(bot, msg, cmd, config).await
                },
            ))
            .branch(Update::filter_callback_query().endpoint(
                |bot: Bot,
                 cq: teloxide::types::CallbackQuery,
                 config: Arc<crate::config::Config>| async move {
                    handler::handle_callback(bot, cq, config).await
                },
            ));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config])
        .error_handler(LoggingErrorHandler::with_custom_text("Bot运行时错误"))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
