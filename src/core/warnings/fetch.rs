use super::cache::{
    alert_cache_path, rss_cache_path, warnings_cache_dir, ItemCacheEntry, RssCache,
};
use super::cap::parse_cap;
use super::polygon::{parse_polygon, point_in_polygon};
use super::rss::parse_rss;
use super::Warning;
use crate::core::error::CuacaError;
use crate::lang::Lang;
use chrono::{DateTime, Duration, Utc};
use reqwest::blocking::Client;
use serde_json;
use std::fs;

/// Extract a stable ID from the CAP link URL.
pub(super) fn cap_id_from_link(link: &str) -> String {
    let path = std::path::Path::new(link);
    let fname = match path.file_name().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return link.to_string(),
    };
    if fname.ends_with("_alert.xml") {
        fname.trim_end_matches("_alert.xml").to_string()
    } else if let Some(dot) = fname.rfind('.') {
        fname[..dot].to_string()
    } else {
        fname.to_string()
    }
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[allow(dead_code)]
fn filter_by_province(
    mut alerts: Vec<Warning>,
    province: &str,
) -> Result<Vec<Warning>, CuacaError> {
    let province_lower = province.to_ascii_lowercase();
    alerts.retain(|w| w.area_desc.to_ascii_lowercase().contains(&province_lower));
    let now = Utc::now();
    alerts.retain(|w| {
        let eff = w.effective.unwrap_or(now);
        let exp = w.expires.unwrap_or(now + Duration::days(1));
        now >= eff && now < exp
    });
    Ok(alerts)
}

/// Fetch active weather warnings for a province, optionally filtered by location.
///
/// Uses RSS feed and CAP alerts with caching. Returns a list of warnings.
/// Errors are propagated as `CuacaError`.
///
/// Note: Callers typically use `unwrap_or_default` to fall back to an empty list on error.
pub fn fetch_warnings(
    province: &str,
    lang: Lang,
    lat: f64,
    lon: f64,
    ttl_minutes: u64,
) -> Result<Vec<Warning>, CuacaError> {
    // Ensure cache directory exists
    let warn_dir = warnings_cache_dir();
    let _ = fs::create_dir_all(&warn_dir);
    let alerts_dir = warn_dir.join("alerts");
    let _ = fs::create_dir_all(&alerts_dir);

    // RSS cache TTL: configurable
    let rss_path = rss_cache_path();
    let rss_fresh = if let Ok(meta) = fs::metadata(&rss_path) {
        if let Ok(modified) = meta.modified() {
            let modified_dt: DateTime<Utc> = modified.into();
            let rss_age = Utc::now().signed_duration_since(modified_dt);
            rss_age < Duration::minutes(ttl_minutes as i64)
        } else {
            false
        }
    } else {
        false
    };

    let rss_items: Vec<ItemCacheEntry> = if rss_fresh {
        let data = fs::read(&rss_path)?;
        let cache = serde_json::from_slice::<RssCache>(&data)?;
        cache.items
    } else {
        // Fresh fetch
        let rss_url = match lang {
            crate::lang::Lang::EN => "https://www.bmkg.go.id/alerts/nowcast/en",
            crate::lang::Lang::ID => "https://www.bmkg.go.id/alerts/nowcast/id",
        };
        let client = Client::new();
        let rss_resp = client.get(rss_url).send()?;
        let rss_bytes = rss_resp.bytes()?;
        let rss_feed = parse_rss(&rss_bytes)?;
        let mut entries = Vec::new();
        for item in rss_feed.channel.item {
            if let Some(ref link) = item.link {
                let id = cap_id_from_link(link);
                entries.push(ItemCacheEntry {
                    link: link.clone(),
                    id,
                });
            }
        }
        // Save RSS cache
        let cache = RssCache {
            fetched_at: Utc::now(),
            items: entries.clone(),
        };
        let _ = fs::create_dir_all(warn_dir);
        if let Ok(mut file) = fs::File::create(&rss_path) {
            let _ = serde_json::to_writer_pretty(&mut file, &cache);
        }
        entries
    };

    // For each item, load or fetch the CAP
    let mut alerts = Vec::new();
    let client = Client::new();
    let now = Utc::now();
    let fallback_ttl = Duration::minutes(ttl_minutes as i64);
    for entry in rss_items {
        let cap_path = alert_cache_path(&entry.id);
        // Try to use cached CAP if it exists and is still valid
        if let Ok(data) = fs::read(&cap_path) {
            if let Ok(warn) = serde_json::from_slice::<Warning>(&data) {
                // Check if still valid: either not expired or within fallback TTL
                if let Some(exp) = warn.expires {
                    if now < exp {
                        alerts.push(warn);
                        continue;
                    }
                } else {
                    if let Ok(meta) = fs::metadata(&cap_path) {
                        if let Ok(modified) = meta.modified() {
                            let modified_dt: DateTime<Utc> = modified.into();
                            let age = now.signed_duration_since(modified_dt);
                            if age < fallback_ttl {
                                alerts.push(warn);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // Fetch CAP
        let cap_resp = match client.get(&entry.link).send() {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        let cap_bytes = match cap_resp.bytes() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let cap = match parse_cap(&cap_bytes) {
            Ok(cap) => cap,
            Err(_) => continue,
        };
        let info = cap.info;
        let effective = parse_iso(&info.effective);
        let expires = parse_iso(&info.expires);
        let mut polygons = Vec::new();
        for p in info.area.polygons {
            let pts = parse_polygon(&p);
            if !pts.is_empty() {
                polygons.push(pts);
            }
        }
        let warning = Warning {
            headline: info.headline,
            area_desc: info.area.area_desc,
            effective,
            expires,
            polygons,
            web: info.web,
        };
        // Cache it
        if let Ok(mut file) = fs::File::create(&cap_path) {
            let _ = serde_json::to_writer_pretty(&mut file, &warning);
        }
        alerts.push(warning);
    }

    // Polygon-based filtering with fallback to area name and time validity
    let point = (lat, lon);
    let mut filtered: Vec<Warning> = alerts
        .iter()
        .filter(|w| {
            !w.polygons.is_empty() && w.polygons.iter().any(|poly| point_in_polygon(point, poly))
        })
        .cloned()
        .collect();

    if filtered.is_empty() {
        filtered = alerts.clone();
        let province_lower = province.to_ascii_lowercase();
        filtered.retain(|w| w.area_desc.to_ascii_lowercase().contains(&province_lower));
        let now = Utc::now();
        filtered.retain(|w| {
            let eff = w.effective.unwrap_or(now);
            let exp = w.expires.unwrap_or(now + Duration::days(1));
            now >= eff && now < exp
        });
    }

    Ok(filtered)
}
