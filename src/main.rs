use teloxide::Bot;
use crate::config::Config;
use tracing::info;

mod config;
mod error;
mod telemetry;
mod bot;
mod models;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = Config::load().expect("配置加载失败");
    telemetry::init_telemetry(&config)?;
    info!("Bot配置加载完成");

    let bot = Bot::new(&config.telegram_token);

    info!("🚀 Bot启动中...");
    bot::run(bot, config).await?;

    Ok(())
}
