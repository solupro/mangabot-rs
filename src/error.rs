use config::ConfigError;
use teloxide::RequestError;
use thiserror::Error;

#[allow(dead_code)]
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

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("压缩错误: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("遍历错误: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("内部错误: {0}")]
    InternalError(String),
}

// 2024: 更优雅的Result别名
pub type Result<T, E = BotError> = std::result::Result<T, E>;
