use std::sync::OnceLock;
use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client, ClientBuilder,
};
use std::time::Duration;
use crate::config::Config;

static HTTP_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Mobile/15E148 Safari/604.1";
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();
static DOWNLOAD_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init(config: &Config) -> crate::error::Result<()> {
    if HTTP_CLIENT.get().is_some() || DOWNLOAD_CLIENT.get().is_some() {
        panic!("`init()` called more than once! HTTP clients are already initialized.");
    }

    let headers = {
        let mut headers = HeaderMap::with_capacity(2); // 预分配容量
        headers.insert(USER_AGENT, HeaderValue::from_static(HTTP_USER_AGENT));
        headers.insert(REFERER, HeaderValue::from_str(&config.manga.base_url)
            .expect("config.base_url contains invalid characters for HTTP header"));
        headers
    };

    let build_client = |timeout_secs: u64, name: &str| -> Client {
        ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .cookie_store(true)
            .default_headers(headers.clone()) // HeaderMap 实现 Clone，开销极小
            .build()
            .unwrap_or_else(|e| panic!("Failed to build {} client: {}", name, e))
    };

    let http_client = build_client(config.server.http_timeout, "HTTP");
    let download_client = build_client(config.server.download_timeout, "DOWNLOAD");

    HTTP_CLIENT.set(http_client).expect("HTTP_CLIENT already set (this should be unreachable)");
    DOWNLOAD_CLIENT.set(download_client).expect("DOWNLOAD_CLIENT already set (this should be unreachable)");

    Ok(())
}
pub fn http() -> &'static Client {
    HTTP_CLIENT.get().expect("HTTP client not initialized — call `init()` first!")
}

pub fn download() -> &'static Client {
    DOWNLOAD_CLIENT.get().expect("DOWNLOAD client not initialized — call `init()` first!")
}
