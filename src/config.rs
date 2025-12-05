use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub server: ServerConfig,
    pub manga: MangaConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    pub bot_name: String,
    pub telegram_token: String,
    pub admin_ids: Vec<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub web_host: String,
    pub http_timeout: u64,
    pub download_timeout: u64,
    pub log_level: String,
    pub log_path: String,
    pub download_path: String,
    pub download_concurrency: usize,
    pub cache_download_token_minute_ttl: u64,
    pub cache_download_token_max_size: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MangaConfig {
    pub base_url: String,
    pub preview_size: u32,
    pub cache_image_minute_ttl: u64,
    pub cache_image_max_size: u64,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::new("config.toml", config::FileFormat::Toml))
            .set_default("bot.bot_name", "mangars_bot")?
            .set_default("bot.telegram_token", "")?
            .set_default("bot.admin_ids", Vec::<i64>::new())?
            .set_default("server.port", 8087)?
            .set_default("server.web_host", "http://localhost:8087")?
            .set_default("server.http_timeout", 10)?
            .set_default("server.download_timeout", 15)?
            .set_default("server.log_level", "info")?
            .set_default("server.log_path", "/tmp/mangabot/app.log")?
            .set_default("server.download_path", "/tmp/mangabot/downloads")?
            .set_default("server.download_concurrency", 5)?
            .set_default("server.cache_download_token_minute_ttl", 10)?
            .set_default("server.cache_download_token_max_size", 256)?
            .set_default("manga.base_url", "")?
            .set_default("manga.preview_size", 10)?
            .set_default("manga.cache_image_minute_ttl", 20)?
            .set_default("manga.cache_image_max_size", 256)?
            .build()?
            .try_deserialize()
    }

    pub fn is_admin(&self, user_id: u64) -> bool {
        self.bot.admin_ids.contains(&user_id)
    }
}

#[test]
fn test_config() {
    let config = Config::load().unwrap();
    assert_eq!(config.server.log_level, "info");
    assert_eq!(config.server.log_path, "/tmp/mangabot/app.log");
    assert_eq!(config.bot.bot_name, "mangars_bot");
}

#[test]
fn test_is_admin() {
    let config = Config::load().unwrap();
    assert!(!config.is_admin(123456789));
}
