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
