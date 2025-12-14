use crate::error::BotError;
use crate::utils::client;
use futures::{StreamExt, stream};
use reqwest::Url;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

fn same_host(url: &str, base_url: &str) -> bool {
    match (Url::parse(url), Url::parse(base_url)) {
        (Ok(u), Ok(b)) => u.host_str() == b.host_str(),
        _ => false,
    }
}

pub async fn fetch(url: &str, base_url: &str) -> Result<String, BotError> {
    if !base_url.is_empty() && !same_host(url, base_url) {
        return Err(BotError::InternalError("SSRF blocked: host not allowed".to_string()));
    }
    let resp = client::http().get(url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(BotError::RequestStatusError(format!("{:?}", status)));
    }
    let text = resp.text().await?;
    Ok(text)
}

pub fn resolve_url(v: &str, base_url: &str) -> String {
    if v.starts_with("http") {
        return v.to_string();
    }

    if v.starts_with("////") {
        return format!("https:{}", v.strip_prefix("//").unwrap_or(v));
    }

    if v.starts_with("//") {
        return format!("https:{}", v);
    }
    if base_url.is_empty() {
        return v.to_string();
    }
    if let Ok(base) = Url::parse(base_url) {
        if let Ok(joined) = base.join(v) {
            return joined.to_string();
        }
    }
    v.to_string()
}

async fn download_file(
    client: &reqwest::Client,
    url: &str,
    save_path: &str,
) -> crate::error::Result<()> {
    let mut attempt = 0u32;
    let response = loop {
        attempt += 1;
        match client.get(url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    break resp;
                }
            }
            Err(_) => {}
        }
        if attempt >= 3 {
            return Err(crate::error::BotError::RequestStatusError(
                "下载失败，超过重试次数".to_string(),
            ));
        }
        let delay = 100 * attempt; // 毫秒
        tokio::time::sleep(std::time::Duration::from_millis(delay.into())).await;
    };

    // url 解析文件名
    let raw = url.split('/').last().unwrap_or(url);
    let filename = crate::utils::fs::sanitize_filename(raw);
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
                download_file(&client, &url, &save_path).await.map_err(|e| {
                    error!("下载失败 {}: {:?}", url, e);
                    (url, e)
                })
            }
        })
        .buffer_unordered(max_concurrent) // 限制并发
        .collect()
        .await;

    // 统计结果
    let success = results.iter().filter(|r| r.is_ok()).count();
    info!("下载完成，成功: {}, 失败: {}", success, results.len() - success);
}
