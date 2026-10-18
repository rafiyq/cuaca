use crate::cache::cache_dir;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ItemCacheEntry {
    pub(super) link: String,
    pub(super) id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct RssCache {
    pub(super) fetched_at: DateTime<Utc>,
    pub(super) items: Vec<ItemCacheEntry>,
}

pub(super) fn warnings_cache_dir() -> PathBuf {
    cache_dir().join("warnings")
}

pub(super) fn rss_cache_path() -> PathBuf {
    warnings_cache_dir().join("rss.json")
}

pub(super) fn alert_cache_path(id: &str) -> PathBuf {
    warnings_cache_dir()
        .join("alerts")
        .join(format!("{}.json", id))
}
