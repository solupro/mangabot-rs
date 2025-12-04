use teloxide::{dptree, Bot};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dispatching::HandlerExt;
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::Update;
use crate::bot::commands::Command;
use std::sync::Arc;

pub mod handler;
pub mod commands;

pub async fn run(bot: Bot, config: crate::config::Config) -> crate::error::Result<()> {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(|bot: Bot, msg: teloxide::types::Message, cmd: Command, config: crate::config::Config| async move {
                    handler::handle_command(bot, msg, cmd, config).await
                })
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(|bot: Bot, cq: teloxide::types::CallbackQuery, config: crate::config::Config| async move {
                    handler::handle_callback(bot, cq, config).await
                })
        );


    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config])
        .error_handler(LoggingErrorHandler::with_custom_text("Bot运行时错误"))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
