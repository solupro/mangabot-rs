use reqwest::Url;
use crate::error::BotError;
use crate::utils::client;

pub async fn fetch(url: &str) -> Result<String, BotError> {
    let resp = client::http().get(url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(BotError::RequestStatusError(format!("{:?}", status)));
    }
    let text = resp
        .text()
        .await
        .map_err(|e| BotError::RequestError(e.into()))?;
    Ok(text)
}

pub fn resolve_url(v: &str, base_url: &str) -> String {
    if v.starts_with("http") { return v.to_string(); }
    if v.starts_with("//") { return format!("https:{}", v); }
    if base_url.is_empty() { return v.to_string(); }
    if let Ok(base) = Url::parse(base_url) {
        if let Ok(joined) = base.join(v) { return joined.to_string(); }
    }
    v.to_string()
}