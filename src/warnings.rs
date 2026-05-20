use chrono::{DateTime, Duration, Utc};
use quick_xml::de::Deserializer;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::cache::cache_dir;

fn parse_polygon(s: &str) -> Vec<(f64, f64)> {
    s.split_whitespace()
        .filter_map(|coord| {
            let parts: Vec<&str> = coord.split(',').collect();
            if parts.len() == 2 {
                let lat = parts[0].parse::<f64>().ok()?;
                let lon = parts[1].parse::<f64>().ok()?;
                Some((lat, lon))
            } else {
                None
            }
        })
        .collect()
}

fn point_in_polygon(point: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let (x, y) = point;
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Warning {
    pub headline: String,
    pub area_desc: String,
    pub effective: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>,
    pub polygons: Vec<Vec<(f64, f64)>>,
    pub web: Option<String>,
}

// RSS feed structures
#[derive(Debug, Deserialize)]
struct RssFeed {
    channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
}

// CAP alert structures
#[derive(Debug, Deserialize)]
#[serde(rename = "alert")]
struct CapAlert {
    info: Info,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct Info {
    headline: String,
    effective: String,
    expires: String,
    area: Area,
    web: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct Area {
    #[serde(rename = "areaDesc")]
    area_desc: String,
    #[serde(rename = "polygon")]
    polygons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItemCacheEntry {
    link: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RssCache {
    fetched_at: DateTime<Utc>,
    items: Vec<ItemCacheEntry>,
}

fn warnings_cache_dir() -> PathBuf {
    cache_dir().join("warnings")
}

fn rss_cache_path() -> PathBuf {
    warnings_cache_dir().join("rss.json")
}

fn alert_cache_path(id: &str) -> PathBuf {
    warnings_cache_dir()
        .join("alerts")
        .join(format!("{}.json", id))
}

fn parse_rss(xml: &[u8]) -> Result<RssFeed, Box<dyn std::error::Error>> {
    let mut deserializer = Deserializer::from_reader(xml);
    let feed = RssFeed::deserialize(&mut deserializer)?;
    Ok(feed)
}

fn parse_cap(xml: &[u8]) -> Result<CapAlert, Box<dyn std::error::Error>> {
    let mut deserializer = Deserializer::from_reader(xml);
    let alert = CapAlert::deserialize(&mut deserializer)?;
    Ok(alert)
}

fn filter_by_province(mut alerts: Vec<Warning>, province: &str) -> Vec<Warning> {
    let province_lower = province.to_ascii_lowercase();
    alerts.retain(|w| w.area_desc.to_ascii_lowercase().contains(&province_lower));
    let now = Utc::now();
    alerts.retain(|w| {
        let eff = w.effective.unwrap_or(now);
        let exp = w.expires.unwrap_or(now + Duration::days(1));
        now >= eff && now < exp
    });
    alerts
}

/// Extract a stable ID from the CAP link URL, e.g., ".../CBT20260520001_alert.xml" -> "CBT20260520001"
fn cap_id_from_link(link: &str) -> String {
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

pub fn fetch_warnings(
    province: &str,
    lang: crate::lang::Lang,
    lat: f64,
    lon: f64,
    ttl_minutes: u64,
) -> Vec<Warning> {
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
        if let Ok(data) = fs::read(&rss_path) {
            if let Ok(cache) = serde_json::from_slice::<RssCache>(&data) {
                cache.items
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        // Fresh fetch
        let rss_url = match lang {
            crate::lang::Lang::EN => "https://www.bmkg.go.id/alerts/nowcast/en",
            crate::lang::Lang::ID => "https://www.bmkg.go.id/alerts/nowcast/id",
        };
        let client = Client::new();
        let rss_resp = match client.get(rss_url).send() {
            Ok(resp) => resp,
            Err(_) => return vec![],
        };
        let rss_bytes = match rss_resp.bytes() {
            Ok(b) => b,
            Err(_) => return vec![],
        };
        let rss_feed = match parse_rss(&rss_bytes) {
            Ok(feed) => feed,
            Err(_) => return vec![],
        };
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

    filtered
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_id_from_link() {
        assert_eq!(
            cap_id_from_link("https://www.bmkg.go.id/alerts/nowcast/id/CBT20260520001_alert.xml"),
            "CBT20260520001"
        );
        assert_eq!(cap_id_from_link("https://example.com/foo.xml"), "foo");
    }

    #[test]
    fn test_parse_cap_sample() {
        let sample = r#"<?xml version="1.0" ?>
<alert xmlns="urn:oasis:names:tc:emergency:cap:1.2">
  <identifier>2.49.0.1.360.0.2026.05.20.01.36.001</identifier>
  <sender>cuaca.ekstrem@bmkg.go.id</sender>
  <sent>2026-05-20T07:55:00+07:00</sent>
  <status>Actual</status>
  <msgType>Alert</msgType>
  <scope>Public</scope>
  <info>
    <language>id</language>
    <category>Met</category>
    <event>Hujan Lebat dan Petir</event>
    <urgency>Immediate</urgency>
    <severity>Moderate</severity>
    <certainty>Observed</certainty>
    <eventCode>
      <valueName>OET:v1.2</valueName>
      <value>OET-194</value>
    </eventCode>
    <effective>2026-05-20T08:05:00+07:00</effective>
    <expires>2026-05-20T10:00:00+07:00</expires>
    <senderName>Badan Meteorologi Klimatologi dan Geofisika</senderName>
    <headline>Hujan Lebat disertai Petir di Banten</headline>
    <description>Hujan lebat...</description>
    <web>https://nowcasting.bmkg.go.id/infografis/CBT/2026/05/20/infografis.jpg</web>
    <contact>06221 196</contact>
    <area>
      <areaDesc>Banten</areaDesc>
      <polygon>-6.024,106.412 -6.031,106.408 -6.030,106.384</polygon>
    </area>
  </info>
</alert>"#;
        let mut deserializer = Deserializer::from_reader(sample.as_bytes());
        let alert: CapAlert = CapAlert::deserialize(&mut deserializer).expect("parse cap");
        assert_eq!(alert.info.headline, "Hujan Lebat disertai Petir di Banten");
        assert_eq!(alert.info.area.area_desc, "Banten");
        assert!(parse_iso(&alert.info.effective).is_some());
        assert!(!alert.info.area.polygons.is_empty());
        assert!(alert.info.web.is_some());
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon((5.0, 5.0), &square));
        assert!(!point_in_polygon((15.0, 5.0), &square));
        // edge: point on edge may be considered inside? Our algorithm works for strict interior; it's fine.
    }

    #[test]
    fn test_parse_polygon() {
        let s = " -6.024,106.412 -6.031,106.408 -6.030,106.384 ";
        let pts = parse_polygon(s);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (-6.024, 106.412));
    }
}
