use std::path::Path;
use std::sync::Arc;
use futures::{stream, StreamExt};
use reqwest::Url;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
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


pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
async fn download_file(client: &reqwest::Client, url: &str, save_path: &str) -> Result<(), BoxError> {

    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("下载失败，状态码: {}", response.status()).into());
    }

    // url 解析文件名
    let filename = url.split('/').last().unwrap_or(url);
    let file_path = format!("{}/{}", save_path, filename);

    // 创建目标文件
    let path = Path::new(file_path.as_str());
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = File::create(path).await?;

    // 流式写入文件
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    Ok(())
}

pub async fn download_batch(urls: Vec<String>, save_path: &str, max_concurrent: usize) {
    let client = Arc::new(client::download());

    let results: Vec<_> = stream::iter(urls)
        .map(|url| {
            let client = Arc::clone(&client);
            async move {
                download_file(&client, &url, &save_path).await
                    .map_err(|e|
                        {
                            error!("下载失败 {}: {:?}", url, e);
                            (url, e)
                        }
                    )
            }
        })
        .buffer_unordered(max_concurrent)  // 限制并发
        .collect()
        .await;

    // 统计结果
    let success = results.iter().filter(|r| r.is_ok()).count();
    info!("下载完成，成功: {}, 失败: {}", success, results.len() - success);
}