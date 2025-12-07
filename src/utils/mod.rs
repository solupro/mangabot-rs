use once_cell::sync::Lazy;
use regex::Regex;

pub mod client;
pub mod http;
pub mod codec;
pub mod dom;
pub mod cache;
pub mod zip;

static NUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"-(\d+)").unwrap());

pub fn extract_num(s: &str) -> Option<i64> {
    NUM_RE
        .captures(s)
        .and_then(|cap| cap.get(1)?.as_str().parse().ok())
}

pub fn digits_to_i32(s: &str) -> i32 {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<i32>()
        .unwrap_or(0)
}

pub fn escape_md_v2(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}