use super::location_remote;
use crate::core::cache::cache_dir;
use crate::core::error::CuacaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_TTL_SECS: u64 = 86400; // 24 hours

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct ResolveCache {
    version: u32,
    gps: HashMap<String, CacheEntry>,
    names: HashMap<String, CacheEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CacheEntry {
    adm4: String,
    epoch_secs: u64,
}

fn cache_file_path() -> PathBuf {
    cache_dir().join("cuaca-resolve.json")
}

fn load_cache() -> ResolveCache {
    let path = cache_file_path();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_else(|_| ResolveCache::default())
    } else {
        ResolveCache::default()
    }
}

fn save_cache(cache: &ResolveCache) -> Result<(), CuacaError> {
    let path = cache_file_path();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn gps_key(lat: f64, lon: f64) -> String {
    format!("{:.4}_{:.4}", lat, lon)
}

fn name_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn is_fresh(epoch_secs: u64) -> bool {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(now) => now.as_secs() < epoch_secs + CACHE_TTL_SECS,
        Err(_) => false,
    }
}

pub fn resolve(
    adm4: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
    name: Option<&str>,
) -> Result<String, CuacaError> {
    if let Some(code) = adm4 {
        return Ok(code.to_string());
    }

    if lat.is_some() && lon.is_none() {
        return Err(CuacaError::Config("--lat requires --lon".to_string()));
    }
    if lon.is_some() && lat.is_none() {
        return Err(CuacaError::Config("--lon requires --lat".to_string()));
    }

    if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
        let key = gps_key(lat_val, lon_val);
        let cache = load_cache();
        if let Some(entry) = cache.gps.get(&key) {
            if is_fresh(entry.epoch_secs) {
                return Ok(entry.adm4.clone());
            }
        }

        let adm4_code = location_remote::fetch_nearest(lat_val, lon_val)?;

        // Update cache
        let mut cache = load_cache();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CuacaError::Unknown(e.to_string()))?
            .as_secs();
        cache.gps.insert(
            key,
            CacheEntry {
                adm4: adm4_code.clone(),
                epoch_secs: now,
            },
        );
        let _ = save_cache(&cache); // best effort

        Ok(adm4_code)
    } else if let Some(name_val) = name {
        let key = name_key(name_val);
        let cache = load_cache();
        if let Some(entry) = cache.names.get(&key) {
            if is_fresh(entry.epoch_secs) {
                return Ok(entry.adm4.clone());
            }
        }

        let adm4_code = location_remote::fetch_search(name_val)?;

        let mut cache = load_cache();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CuacaError::Unknown(e.to_string()))?
            .as_secs();
        cache.names.insert(
            key,
            CacheEntry {
                adm4: adm4_code.clone(),
                epoch_secs: now,
            },
        );
        let _ = save_cache(&cache);

        Ok(adm4_code)
    } else {
        Err(CuacaError::Location(
            "provide --adm4, --lat/--lon, or --name".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::location_remote::reset_breaker;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn setup_temp_cache() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CUACA_CACHE_DIR", dir.path());
        dir
    }

    #[test]
    fn test_gps_key_format() {
        let key = gps_key(10.123456, 110.987654);
        assert_eq!(key, "10.1235_110.9877"); // rounded to 4 decimals
    }

    #[test]
    fn test_name_key_normalization() {
        assert_eq!(name_key("  Kemayoran  "), "kemayoran");
        assert_eq!(name_key("JAKARTA"), "jakarta");
    }

    #[test]
    #[serial]
    fn test_resolve_gps_cache_hit() {
        let _temp = setup_temp_cache();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cache = ResolveCache {
            version: 1,
            gps: {
                let mut map = HashMap::new();
                map.insert(
                    gps_key(10.0, 110.0),
                    CacheEntry {
                        adm4: "31.71.03.1001".to_string(),
                        epoch_secs: now + 10000, // future, definitely fresh
                    },
                );
                map
            },
            names: HashMap::new(),
        };
        let cache_file = cache_file_path();
        if let Some(parent) = cache_file.parent() {
            let _ = create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(&cache).unwrap();
        std::fs::write(&cache_file, content).unwrap();

        let result = resolve(None, Some(10.0), Some(110.0), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "31.71.03.1001");
    }

    #[test]
    #[serial]
    fn test_resolve_gps_cache_miss_calls_remote() {
        let _temp = setup_temp_cache();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/nearest?lat=10&lon=110&limit=1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[{"code":"33.12.34.2002"}]}"#)
            .create();

        reset_breaker();
        env::set_var("WILAYAH_API_BASE", &server.url());
        let result = resolve(None, Some(10.0), Some(110.0), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "33.12.34.2002");
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_resolve_name_cache_miss_and_save() {
        let _temp = setup_temp_cache();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/search?q=kemayoran&limit=10")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[{"code":"31.71.03.1001"}]}"#)
            .create();

        reset_breaker();
        env::set_var("WILAYAH_API_BASE", &server.url());
        let result = resolve(None, None, None, Some("kemayoran"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "31.71.03.1001");
        mock.assert();

        // Verify cache contains the entry
        let cache = load_cache();
        let key = name_key("kemayoran");
        assert!(cache.names.contains_key(&key));
        let entry = cache.names.get(&key).unwrap();
        assert_eq!(entry.adm4, "31.71.03.1001");
    }

    #[test]
    fn test_resolve_adm4_direct() {
        let result = resolve(Some("31.71.03.1001"), None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "31.71.03.1001");
    }

    #[test]
    fn test_resolve_lat_without_lon_error() {
        let result = resolve(None, Some(1.0), None, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            CuacaError::Config(msg) => assert!(msg.contains("--lat requires --lon")),
            _ => panic!("wrong error type"),
        }
    }

    #[test]
    fn test_resolve_lon_without_lat_error() {
        let result = resolve(None, None, Some(1.0), None);
        assert!(result.is_err());
        match result.unwrap_err() {
            CuacaError::Config(msg) => assert!(msg.contains("--lon requires --lat")),
            _ => panic!("wrong error type"),
        }
    }

    #[test]
    fn test_resolve_no_args_error() {
        let result = resolve(None, None, None, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            CuacaError::Location(msg) => assert!(msg.contains("provide --adm4")),
            _ => panic!("wrong error type"),
        }
    }
}
