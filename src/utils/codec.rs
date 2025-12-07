use crate::bot::commands::Command;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use std::fmt;
use teloxide::types::InlineKeyboardButton;

/// 支持的命令参数类型（可扩展）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CommandArg {
    String(String),
    I32(i32),
    I64(i64),
    Bool(bool),
    // 可添加: Float(f64), Json(serde_json::Value) 等
}

impl From<&str> for CommandArg {
    fn from(s: &str) -> Self {
        CommandArg::String(s.to_string())
    }
}

impl From<String> for CommandArg {
    fn from(s: String) -> Self {
        CommandArg::String(s)
    }
}

impl From<i32> for CommandArg {
    fn from(v: i32) -> Self {
        CommandArg::I32(v)
    }
}

impl From<i64> for CommandArg {
    fn from(v: i64) -> Self {
        CommandArg::I64(v)
    }
}

impl From<bool> for CommandArg {
    fn from(v: bool) -> Self {
        CommandArg::Bool(v)
    }
}

impl fmt::Display for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandArg::String(s) => write!(f, "{}", s),
            CommandArg::I32(v) => write!(f, "{}", v),
            CommandArg::I64(v) => write!(f, "{}", v),
            CommandArg::Bool(v) => write!(f, "{}", if *v { "1" } else { "0" }),
        }
    }
}

/// 命令载体（带版本，便于未来扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncodedCommand {
    v: u8,                 // 版本号
    cmd: String,           // 命令名（如 "rank"）
    args: Vec<CommandArg>, // 参数列表
}

// ================
// 2. 编码函数
// ================

/// 将命令 + 参数编码为 Telegram-safe 字符串（用于 /start payload）
///
/// # 示例
/// ```
/// let payload = encode_command("rank", &["day", 123]).unwrap();
/// // payload = "cmFuazpkYXk6MTIz" (base64 of "rank:day:123")
/// ```
pub fn encode_command(
    command: &str,
    args: &[impl Into<CommandArg> + Clone],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 1. 构造数据
    let cmd = EncodedCommand {
        v: 1,
        cmd: command.to_string(),
        args: args.iter().cloned().map(|a| a.into()).collect(),
    };

    // 2. 序列化为紧凑字符串（"cmd:arg1:arg2"）
    let plain = {
        let mut parts = vec![cmd.cmd];
        for arg in &cmd.args {
            parts.push(arg.to_string());
        }
        parts.join(":")
    };

    // 3. Base64URL 编码（无 +/，无 = 填充）
    let encoded = URL_SAFE_NO_PAD.encode(plain.as_bytes());

    // 4. Telegram 限制: start payload ≤ 64 字符
    if encoded.len() > 64 {
        return Err::<String, Box<dyn std::error::Error + Send + Sync>>(
            "Encoded command exceeds 64 characters".into(),
        );
    }

    Ok(encoded)
}

pub async fn decode_command(
    payload: &str,
) -> Result<Command, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Base64 解码
    let bytes = URL_SAFE_NO_PAD.decode(payload)?;
    let plain = String::from_utf8(bytes)?;

    // 2. 拆分 "cmd:arg1:arg2"
    let parts: Vec<&str> = plain.split(':').collect();
    if parts.is_empty() {
        return Err::<Command, Box<dyn std::error::Error + Send + Sync>>("Empty payload".into());
    }

    let command = parts[0];
    let args = parts[1..]
        .iter()
        .map(|&s| parse_arg(s))
        .collect::<Result<Vec<_>, _>>()?;

    let cmd = match command.to_lowercase().as_str() {
        "rank" => {
            // 兼容 /rank [period] [page] 的解析逻辑
            // args[0] 为数字则视为 page，否则视为 period
            let (period, page) = match args.get(0) {
                Some(CommandArg::I32(pg)) => (None, Some(*pg)),
                Some(CommandArg::I64(pg)) => {
                    let pg_i32 = i32::try_from(*pg).ok();
                    (None, pg_i32)
                }
                Some(CommandArg::String(p)) => {
                    let period = Some(p.clone());
                    let page = args.get_i32(1);
                    (period, page)
                }
                _ => (None, None),
            };
            Command::Rank(period, page)
        }
        "csearch" => {
            let cache_num = if parts.len() > 1 {
                parts[1].parse::<u64>().ok()
            } else {
                None
            };
            let typ = if parts.len() > 2 {
                Some(parts[2].to_string())
            } else {
                None
            };
            let page = if parts.len() > 3 {
                parts[3].parse::<i32>().ok()
            } else {
                Some(1)
            };

            let num = cache_num.expect("缓存编号不能为空");
            let key = super::cache::search_num_to_key(num).await;

            Command::Search(key, typ, page)
        }
        "info" => {
            let aid = if parts.len() > 1 {
                parts[1].to_string()
            } else {
                String::new()
            };
            Command::Info(aid)
        }
        "preview" => {
            let aid = if args.len() > 0 {
                args.get_i64(0)
            } else {
                Some(0)
            };
            let page = if args.len() > 1 {
                args.get_i32(1)
            } else {
                Some(1)
            };

            Command::Preview(aid.map(|s| s.to_string()), page)
        }
        "zip" => {
            let aid = if parts.len() > 1 {
                parts[1].parse::<i64>().unwrap_or(0)
            } else {
                0
            };
            Command::Zip(aid)
        }
        "cate" => {
            let cate = if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            };
            let sub = if parts.len() > 2 {
                Some(parts[2].to_string())
            } else {
                None
            };
            let page = if parts.len() > 3 {
                parts[3].parse::<i32>().ok()
            } else {
                Some(1)
            };
            Command::Cate(cate, sub, page)
        }
        _ => Command::Start(None),
    };

    Ok(cmd)
}

// ================
// 4. 辅助函数
// ================

/// 尝试将字符串解析为 CommandArg（保守策略）
fn parse_arg(s: &str) -> Result<CommandArg, Box<dyn std::error::Error + Send + Sync>> {
    // 优先尝试 i64（覆盖 i32）
    if let Ok(v) = s.parse::<i64>() {
        return Ok(CommandArg::I64(v));
    }

    // 尝试 bool: "1"/"0" 或 "true"/"false"
    if s == "1" || s.eq_ignore_ascii_case("true") {
        return Ok(CommandArg::Bool(true));
    }
    if s == "0" || s.eq_ignore_ascii_case("false") {
        return Ok(CommandArg::Bool(false));
    }

    // 默认为字符串
    Ok(CommandArg::String(s.to_string()))
}

// ================
// 5. 实用扩展 trait
// ================

/// 方便从 Vec<CommandArg> 提取类型安全参数
pub trait CommandArgsExt {
    fn get_string(&self, index: usize) -> Option<&str>;
    fn get_i32(&self, index: usize) -> Option<i32>;
    fn get_i64(&self, index: usize) -> Option<i64>;
    fn get_bool(&self, index: usize) -> Option<bool>;
}

impl CommandArgsExt for Vec<CommandArg> {
    fn get_string(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(|arg| match arg {
            CommandArg::String(s) => Some(s.as_str()),
            _ => None,
        })
    }

    fn get_i32(&self, index: usize) -> Option<i32> {
        self.get(index).and_then(|arg| match arg {
            CommandArg::I32(v) => Some(*v),
            CommandArg::I64(v) if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 => Some(*v as i32),
            _ => None,
        })
    }

    fn get_i64(&self, index: usize) -> Option<i64> {
        self.get(index).and_then(|arg| match arg {
            CommandArg::I64(v) => Some(*v),
            CommandArg::I32(v) => Some(*v as i64),
            _ => None,
        })
    }

    fn get_bool(&self, index: usize) -> Option<bool> {
        self.get(index).and_then(|arg| match arg {
            CommandArg::Bool(v) => Some(*v),
            _ => None,
        })
    }
}

pub fn encode_command_button(
    text: &str,
    command: &str,
    args: &[impl Into<CommandArg> + Clone],
) -> InlineKeyboardButton {
    let data = encode_command(command, &args).unwrap();
    InlineKeyboardButton::callback(text, data)
}

pub fn encode_command_link(
    bot_name: &str,
    command: &str,
    args: &[impl Into<CommandArg> + Clone],
) -> String {
    let data = encode_command(command, &args).unwrap();
    format!("https://t.me/{}?start={}", bot_name, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_command() {
        let cmd = encode_command("search", &["123456789011", "user_nicename", "1"]).unwrap();
        println!("{:?}", cmd);
    }
}
