use chrono::{DateTime, Duration, Utc};
use quick_xml::de::Deserializer;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Warning {
    pub headline: String,
    pub area_desc: String,
    pub effective: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct Area {
    #[serde(rename = "areaDesc")]
    area_desc: String,
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

fn cache_path() -> PathBuf {
    std::env::temp_dir().join("cuaca-warnings.json")
}

#[derive(Debug, Serialize, Deserialize)]
struct WarningsCache {
    fetched_at: DateTime<Utc>,
    alerts: Vec<Warning>,
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

pub fn fetch_warnings(province: &str, lang: crate::lang::Lang) -> Vec<Warning> {
    // Attempt to load fresh cache (5 minutes)
    let cache_file = cache_path();
    if let Ok(data) = fs::read(&cache_file) {
        if let Ok(cache) = serde_json::from_slice::<WarningsCache>(&data) {
            if (Utc::now() - cache.fetched_at) < Duration::minutes(5) {
                return filter_by_province(cache.alerts.clone(), province);
            }
        }
    }

    // Fetch fresh data
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
    let items = match parse_rss(&rss_bytes) {
        Ok(feed) => feed.channel.item,
        Err(_) => return vec![],
    };

    let mut alerts = Vec::new();
    for item in items {
        let link = match item.link {
            Some(ref l) => l,
            None => continue,
        };
        let cap_resp = match client.get(link).send() {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        let cap_bytes = match cap_resp.bytes() {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Ok(cap) = parse_cap(&cap_bytes) {
            let info = cap.info;
            let effective = parse_iso(&info.effective);
            let expires = parse_iso(&info.expires);
            alerts.push(Warning {
                headline: info.headline,
                area_desc: info.area.area_desc,
                effective,
                expires,
            });
        }
    }

    // Save to cache
    let cache = WarningsCache {
        fetched_at: Utc::now(),
        alerts: alerts.clone(),
    };
    if let Ok(mut file) = fs::File::create(&cache_file) {
        let _ = serde_json::to_writer_pretty(&mut file, &cache);
    }

    filter_by_province(alerts, province)
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
      <polygon>-6.024,106.412 -6.031,106.408</polygon>
    </area>
  </info>
</alert>"#;
        let mut deserializer = Deserializer::from_reader(sample.as_bytes());
        let alert: CapAlert = CapAlert::deserialize(&mut deserializer).expect("parse cap");
        assert_eq!(alert.info.headline, "Hujan Lebat disertai Petir di Banten");
        assert_eq!(alert.info.area.area_desc, "Banten");
        assert!(parse_iso(&alert.info.effective).is_some());
    }
}
