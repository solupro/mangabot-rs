use crate::config::Config;
use moka::future::Cache;
use std::sync::OnceLock;
use std::time::Duration;

static IMAGE_CACHE: OnceLock<Cache<String, Vec<String>>> = OnceLock::new();
static DOWNLOAD_TOKEN_CACHE: OnceLock<Cache<String, String>> = OnceLock::new();

pub fn init(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    fn build_cache<K, V>(ttl_minutes: u64, max_capacity: u64) -> Cache<K, V>
    where
        K: std::hash::Hash + Eq + Send + Sync + std::fmt::Debug + 'static,
        V: Send + Sync + std::fmt::Debug + Clone + 'static,
    {
        Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_mins(ttl_minutes)) // 修复2：from_secs 替代 from_mins
            .eviction_listener(|key, _value, cause| {
                tracing::info!(?key, ?cause, "缓存项被驱逐");
            })
            .build()
    }

    let image_cache: Cache<String, Vec<String>> = build_cache(
        config.manga.cache_image_minute_ttl,
        config.manga.cache_image_max_size,
    );

    let download_token_client: Cache<String, String> = build_cache(
        config.server.cache_download_token_minute_ttl,
        config.server.cache_download_token_max_size,
    );

    IMAGE_CACHE.set(image_cache).expect("IMAGE_CACHE init failed");
    DOWNLOAD_TOKEN_CACHE.set(download_token_client).expect("DOWNLOAD_TOKEN_CACHE init failed");

    Ok(())
}

pub fn image_cache() -> &'static Cache<String, Vec<String>> {
    IMAGE_CACHE.get().expect("IMAGE_CACHE not initialized")
}

pub fn download_token_cache() -> &'static Cache<String, String> {
    DOWNLOAD_TOKEN_CACHE.get().expect("DOWNLOAD_TOKEN_CACHE not initialized")
}