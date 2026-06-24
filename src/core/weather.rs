use crate::core::error::CuacaError;
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheWrapper {
    pub fetched_at: String, // ISO 8601 UTC
    pub adm4: String,
    pub forecast: Value,
}

impl CacheWrapper {
    pub fn is_fresh(&self, ttl_secs: u64) -> bool {
        if let Ok(dt) = DateTime::parse_from_rfc3339(&self.fetched_at) {
            let now = Utc::now();
            let age = now.signed_duration_since(dt);
            age.num_seconds() < ttl_secs as i64
        } else {
            false
        }
    }
}

/// Fetch weather with exponential backoff. No caching.
pub fn fetch_weather(client: &Client, url: &str) -> Result<Value, CuacaError> {
    let mut iterations = 0;
    loop {
        match client.get(url).send() {
            Ok(response) => match response.json::<Value>() {
                Ok(json) => return Ok(json),
                Err(e) => return Err(CuacaError::Data(format!("JSON parse error: {}", e))),
            },
            Err(e) => {
                iterations += 1;
                if iterations >= 20 {
                    return Err(CuacaError::Network(e));
                }
                thread::sleep(Duration::from_millis(500 * iterations as u64));
            }
        }
    }
}

/// Load cache wrapper from file.
fn load_wrapper(path: &PathBuf) -> Result<Option<CacheWrapper>, CuacaError> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)?;
    let wrapper: CacheWrapper = serde_json::from_str(&data)?;
    Ok(Some(wrapper))
}

/// Save cache wrapper to file.
fn save_wrapper(path: &PathBuf, wrapper: &CacheWrapper) -> Result<(), CuacaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(wrapper)?;
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Append wrapper to archive JSONL if archive enabled.
fn maybe_archive(wrapper: &CacheWrapper, archive: bool) -> Result<(), CuacaError> {
    if !archive {
        return Ok(());
    }
    let cache_dir = crate::core::cache::cache_dir();
    let archive_path = cache_dir.join("forecasts.jsonl");
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(archive_path)?;
    let line = serde_json::to_string(wrapper)? + "\n";
    file.write_all(line.as_bytes())?;
    Ok(())
}

/// Ensure we have a fresh forecast for the given adm4.
/// Returns the forecast JSON (Value). Will fetch from BMKG if cache missing or stale.
pub fn ensure_forecast(adm4: &str, ttl_secs: u64, archive: bool) -> Result<Value, CuacaError> {
    let cache_dir = crate::core::cache::cache_dir();
    let cachefile = cache_dir.join(format!("cuaca-{}.json", adm4));

    // Try load cache
    if let Ok(Some(wrapper)) = load_wrapper(&cachefile) {
        if wrapper.is_fresh(ttl_secs) {
            return Ok(wrapper.forecast);
        }
    }

    // Fetch fresh
    let client = Client::new();
    let url = format!(
        "https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4={}",
        adm4
    );
    let forecast = fetch_weather(&client, &url)?;

    // Build wrapper
    let fetched_at = Utc::now().to_rfc3339();
    let wrapper = CacheWrapper {
        fetched_at,
        adm4: adm4.to_string(),
        forecast: forecast.clone(),
    };

    // Save active cache
    let _ = save_wrapper(&cachefile, &wrapper); // best effort

    // Archive if enabled
    let _ = maybe_archive(&wrapper, archive); // best effort

    Ok(forecast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cache::cache_dir;
    use std::fs;

    #[test]
    fn test_cache_wrapper_serialize_deserialize() {
        let wrapper = CacheWrapper {
            fetched_at: "2025-06-06T00:00:00Z".to_string(),
            adm4: "1234".to_string(),
            forecast: serde_json::json!({"test": true}),
        };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        let deserialized: CacheWrapper = serde_json::from_str(&serialized).unwrap();
        assert_eq!(wrapper.fetched_at, deserialized.fetched_at);
        assert_eq!(wrapper.adm4, deserialized.adm4);
        assert_eq!(wrapper.forecast, deserialized.forecast);
    }

    #[test]
    fn test_load_wrapper_missing() {
        let cache_dir = cache_dir();
        let path = cache_dir.join("nonexistent_cache.json");
        let result = load_wrapper(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_and_load_wrapper() {
        let cache_dir = cache_dir();
        let path = cache_dir.join("test_cache_weather.json");
        let wrapper = CacheWrapper {
            fetched_at: Utc::now().to_rfc3339(),
            adm4: "5678".to_string(),
            forecast: serde_json::json!({"data": [1, 2, 3]}),
        };
        save_wrapper(&path, &wrapper).unwrap();
        let loaded = load_wrapper(&path).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.adm4, wrapper.adm4);
        assert_eq!(loaded.forecast, wrapper.forecast);
        // Clean up
        let _ = fs::remove_file(&path);
    }
}
