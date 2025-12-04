use config::ConfigError;
use thiserror::Error;
use teloxide::RequestError;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Telegram API 错误: {0}")]
    Telegram(#[from] RequestError),

    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),

    #[error("权限不足: 需要 {required} 权限")]
    PermissionDenied { required: String },

    #[error("命令参数无效: {reason}")]
    InvalidCommand { reason: String },

    #[error("限流: 请等待 {secs} 秒")]
    RateLimited { secs: u64 },

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("请求错误: {0}")]
    RequestStatusError(String),
}

// 2024: 更优雅的Result别名
pub type Result<T, E = BotError> = std::result::Result<T, E>;