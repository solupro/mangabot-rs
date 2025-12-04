use once_cell::sync::Lazy;
use regex::Regex;
use base64::Engine as _;
pub mod client;
pub mod http;
pub mod codec;

static NUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"-(\d+)").unwrap());

pub fn extract_num(s: &str) -> Option<i64> {
    NUM_RE
        .captures(s)
        .and_then(|cap| cap.get(1)?.as_str().parse().ok())
}
