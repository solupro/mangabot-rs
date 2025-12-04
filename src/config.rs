use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub bot_name: String,
    pub telegram_token: String,
    pub base_url: String,
    pub admin_ids: Vec<i64>,
    pub log_level: String,
    pub log_path: String,
    pub http_timeout: u64,
    pub download_timeout: u64,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::new("config.toml", config::FileFormat::Toml))
            .set_default("bot_name", "")?
            .set_default("telegram_token", "")?
            .set_default("base_url", "")?
            .set_default("admin_ids", Vec::<i64>::new())?
            .set_default("log_level", "info")?
            .set_default("log_path", "/tmp/mangabot.log")?
            .set_default("http_timeout", 10)?
            .set_default("download_timeout", 15)?
            .build()?
            .try_deserialize()
    }

    pub fn is_admin(&mut self, user_id: i64) -> bool {
        self.admin_ids.contains(&user_id)
    }
}

#[test]
fn test_config() {
    let config = Config::load().unwrap();
    // assert_eq!(config.telegram_token, "your_telegram_token");
    // assert_eq!(config.base_url, "your_base_url");
    // assert_eq!(config.admin_ids, vec![123]);
    assert_eq!(config.log_level, "info");
    assert_eq!(config.log_path, "/tmp/mangabot.log");
}

#[test]
fn test_is_admin() {
    let mut config = Config::load().unwrap();
    assert!(!config.is_admin(123456789));
}
