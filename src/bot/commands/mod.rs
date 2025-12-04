use teloxide::utils::command::BotCommands;
use teloxide::utils::command::ParseError;

fn parse_rank_command(s: String) -> Result<(Option<String>, Option<i32>), ParseError> {
    let mut args = s.split_whitespace();

    let period = args.next().map(|s| s.to_string());
    let page = args.next().and_then(|s| s.parse().ok());

    let (final_period, final_page) = match (&period, page) {
        (Some(p), None) if p.parse::<i32>().is_ok() => (None, p.parse().ok()),
        _ => (period, page),
    };

    Ok((final_period, final_page))
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "可用命令:")]
pub enum Command {
    #[command(description = "开始对话", parse_with = parse_start_payload)]
    Start(Option<String>),

    #[command(description = "复制消息")]
    Copy(String),

    #[command(
        description = "排行榜：/rank [period] [page]\n\
                   period: day（默认）, week, month\n\
                   page: 页码（默认 1）",
        parse_with = parse_rank_command
    )]
    Rank(Option<String>, Option<i32>),

    #[command(description = "查询漫画信息: /info <漫画id>")]
    Info(String),
}

pub mod copy;
pub mod start;

pub mod rank;

pub mod info;
fn parse_start_payload(s: String) -> Result<(Option<String>,), ParseError> {
    let s = s.trim();
    if s.is_empty() {
        Ok((None,))
    } else {
        Ok((Some(s.to_string()),))
    }
}
