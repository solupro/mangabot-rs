use teloxide::utils::command::BotCommands;
use teloxide::utils::command::ParseError;

fn parse_string_i32(s: String) -> Result<(Option<String>, Option<i32>), ParseError> {
    let mut args = s.split_whitespace();

    let period = args.next().map(|s| s.to_string());
    let page = args.next().and_then(|s| s.parse().ok());

    let (final_period, final_page) = match (&period, page) {
        (Some(p), None) if p.parse::<i32>().is_ok() => (None, p.parse().ok()),
        _ => (period, page),
    };

    Ok((final_period, final_page))
}

fn parse_string_string_i32(s: String) -> Result<(Option<String>, Option<String>, Option<i32>), ParseError> {
    let mut args = s.split_whitespace();

    let cate = args.next().map(|s| s.to_string());
    let sub = args.next().map(|s| s.to_string());
    let page: Option<i32> = args.next().and_then(|s| s.parse().ok());

    Ok((cate, sub, page))
}

fn parse_start_payload(s: String) -> Result<(Option<String>,), ParseError> {
    let s = s.trim();
    if s.is_empty() {
        Ok((None,))
    } else {
        Ok((Some(s.to_string()),))
    }
}

pub fn build_images_url(base_url: &str, aid: &str) -> String {
    format!(
        "{}/photos-webp-aid-{}.html",
        base_url.trim_end_matches('/'), // 防止双斜杠
        aid
    )
}


#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "可用命令:")]
pub enum Command {
    #[command(description = "开始对话", parse_with = parse_start_payload)]
    Start(Option<String>),

    #[command(description = "搜索 /search <key> <type> <page>", parse_with = parse_string_string_i32)]
    Search(Option<String>, Option<String>, Option<i32>),

    #[command(
        description = "排行榜：/rank <period> <page>\n\
                   period: day（默认）, week, month\n\
                   page: 页码（默认 1）",
        parse_with = parse_string_i32
    )]
    Rank(Option<String>, Option<i32>),

    #[command(description = "分类查询：/cate <category> <subcategory> <page>\n\
                   category: 漫画分类（默认 同人志）\n\
                   subcategory: 子分类（默认 汉化）\n\
                   page: 页码（默认 1）",
              parse_with = parse_string_string_i32)]
    Cate(Option<String>, Option<String>, Option<i32>),

    #[command(description = "查询漫画信息: /info <aid>")]
    Info(String),

    #[command(description = "预览漫画: /preview <aid> <page>", parse_with = parse_string_i32)]
    Preview(Option<String>, Option<i32>),

    #[command(description = "下载漫画: /zip <aid>")]
     Zip(i64),
}

pub mod search;
pub mod start;
pub mod rank;
pub mod info;
pub mod preview;
pub mod zip;
pub mod cate;
