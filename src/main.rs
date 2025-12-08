#![forbid(unsafe_code)]
use teloxide::Bot;
use crate::config::Config;
use tracing::info;

mod config;
mod error;
mod telemetry;
mod bot;
mod models;
mod services;
mod utils;

#[tokio::main]
async fn main() -> crate::error::Result<()> {

    let config = Config::load().expect("配置加载失败");
    telemetry::init_telemetry(&config)?;
    info!("Bot配置加载完成");

    utils::client::init(&config)?;
    info!("HTTP客户端初始化完成");

    utils::cache::init(&config)?;
    info!("缓存初始化完成");

    {
        let config_clone = config.clone();
        if let Err(e) = services::web::start(config_clone) {
            tracing::error!(error = %e, "web server failed");
        }
    }

    let bot = Bot::new(&config.bot.telegram_token);
    info!("🚀 Bot启动中...");
    bot::run(bot, config).await?;

    Ok(())
}
