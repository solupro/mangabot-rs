use std::sync::atomic::{AtomicU64, Ordering};
use crate::config::Config;
use moka::future::Cache;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::info;
use crate::models::MangaDetail;

static IMAGE_CACHE: OnceLock<Cache<String, Vec<String>>> = OnceLock::new();
static INFO_CACHE: OnceLock<Cache<String, MangaDetail>> = OnceLock::new();
static DOWNLOAD_TOKEN_CACHE: OnceLock<Cache<String, String>> = OnceLock::new();

static SEARCH_KEY_NUM_CACHE: OnceLock<Cache<String, u64>> = OnceLock::new();
static SEARCH_NUM_KEY_CACHE: OnceLock<Cache<u64, String>> = OnceLock::new();

static COUNTER: OnceLock<AtomicU64> = OnceLock::new();
static MAX_SEARCH_KEY_NUM: OnceLock<u64> = OnceLock::new();


pub fn init(config: &Config) -> crate::error::Result<()> {
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

    let info_cache: Cache<String, MangaDetail> = build_cache(
        config.manga.cache_info_minute_ttl,
        config.manga.cache_info_max_size,
    );

    let download_token_client: Cache<String, String> = build_cache(
        config.server.cache_download_token_minute_ttl,
        config.server.cache_download_token_max_size,
    );

    let search_key_num_cache: Cache<String, u64> = build_cache(
        config.server.cache_search_key_num_minute_ttl,
        config.server.cache_search_key_num_max_size,
    );

    let search_num_key_cache: Cache<u64, String> = build_cache(
        config.server.cache_search_key_num_minute_ttl,
        config.server.cache_search_key_num_max_size,
    );

    IMAGE_CACHE.set(image_cache).expect("IMAGE_CACHE init failed");
    INFO_CACHE.set(info_cache).expect("INFO_CACHE init failed");
    DOWNLOAD_TOKEN_CACHE.set(download_token_client).expect("DOWNLOAD_TOKEN_CACHE init failed");
    SEARCH_KEY_NUM_CACHE.set(search_key_num_cache).expect("SEARCH_KEY_NUM_CACHE init failed");
    SEARCH_NUM_KEY_CACHE.set(search_num_key_cache).expect("SEARCH_NUM_KEY_CACHE init failed");
    COUNTER.set(AtomicU64::new(0)).expect("COUNTER init failed");
    MAX_SEARCH_KEY_NUM.set(config.server.cache_search_key_num_max_size).expect("MAX_SEARCH_KEY_NUM init failed");

    Ok(())
}

pub fn image_cache() -> &'static Cache<String, Vec<String>> {
    IMAGE_CACHE.get().expect("IMAGE_CACHE not initialized")
}

pub fn info_cache() -> &'static Cache<String, MangaDetail> {
    INFO_CACHE.get().expect("INFO_CACHE not initialized")
}

pub fn download_token_cache() -> &'static Cache<String, String> {
    DOWNLOAD_TOKEN_CACHE.get().expect("DOWNLOAD_TOKEN_CACHE not initialized")
}

fn increment_cyclic() -> u64 {
    let counter = COUNTER.get().expect("COUNTER not initialized");
    let max = MAX_SEARCH_KEY_NUM.get().expect("MAX_SEARCH_KEY_NUM not initialized");
    loop {
        let current = counter.load(Ordering::Relaxed);
        let next = if current >= *max {
            1
        } else {
            current + 1
        };

        // 尝试原子更新，如果期间值被其他线程修改则重试
        match counter.compare_exchange_weak(
            current,
            next,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            Ok(_) => return next,
            Err(_) => continue, // 竞争失败，重新尝试
        }
    }
}

pub async fn search_key_to_num(key: &str) -> u64 {
    let key_cache = SEARCH_KEY_NUM_CACHE.get();
    if key_cache.is_none() {
        return 0;
    }
    let key_cache = key_cache.unwrap();
    if let Some(num) = key_cache.get(key).await {
        num
    } else {
        let num = increment_cyclic();
        key_cache.insert(key.to_string(), num).await;
        if let Some(num_cache) = SEARCH_NUM_KEY_CACHE.get() {
            num_cache.insert(num, key.to_string()).await;
        }
        info!("新增搜索键 {} 对应编号 {}", key, num);
        num
    }
}

pub async fn search_num_to_key(num: u64) -> Option<String> {
    let num_cache: Option<&Cache<u64, String>> = SEARCH_NUM_KEY_CACHE.get();
    if num_cache.is_none() {
        return None;
    }
    let num_cache = num_cache.unwrap();
    num_cache.get(&num).await
}
