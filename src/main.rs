use std::collections::HashMap;
use std::fs::{metadata, read_to_string, File};
use std::io::Write;
use std::process::exit;
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::{Locale, NaiveDateTime};
use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::cli::Args;
use crate::format::{
    format_indicator, format_temp, format_time, format_wind_dir_icon, get_weather_icon,
};

mod cli;
mod constants;
mod format;
mod lang;

#[derive(Deserialize)]
struct GpsCache {
    adm4: String,
    lat: f64,
    lon: f64,
    epoch_secs: u64,
}

impl GpsCache {
    fn save(&self, path: &str) {
        if let Ok(mut f) = File::create(path) {
            let _ = f.write_all(
                serde_json::to_string_pretty(&json!({
                    "adm4": self.adm4,
                    "lat": self.lat,
                    "lon": self.lon,
                    "epoch_secs": self.epoch_secs
                }))
                .unwrap()
                .as_bytes(),
            );
        }
    }

    fn is_stale(&self, max_age_secs: u64) -> bool {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() > self.epoch_secs + max_age_secs)
            .unwrap_or(true)
    }
}

fn resolve_adm4(args: &Args) -> String {
    const GPS_CACHE_FILE: &str = "/tmp/cuaca-gps.json";
    const GPS_CACHE_MAX_AGE_SECS: u64 = 86400;

    if let Some(ref code) = args.adm4 {
        return code.clone();
    }

    if let (Some(lat), Some(lon)) = (args.lat, args.lon) {
        let cache_valid = read_to_string(GPS_CACHE_FILE)
            .ok()
            .and_then(|s| serde_json::from_str::<GpsCache>(&s).ok())
            .map(|c| {
                if c.lat == lat && c.lon == lon && !c.is_stale(GPS_CACHE_MAX_AGE_SECS) {
                    Some(c.adm4)
                } else {
                    None
                }
            })
            .flatten();

        if let Some(adm4) = cache_valid {
            return adm4;
        }

        let conn = wilayah::open().unwrap_or_else(|e| {
            eprintln!(
                "{{\"text\":\"⛔️\", \"tooltip\":\"failed to open location db: {}\"}}",
                e
            );
            exit(0)
        });

        let results = wilayah::find_nearest(&conn, lat, lon, 1).unwrap_or_else(|e| {
            eprintln!(
                "{{\"text\":\"⛔️\", \"tooltip\":\"location lookup failed: {}\"}}",
                e
            );
            exit(0)
        });

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(village) = results.into_iter().next() {
            GpsCache {
                adm4: village.code.clone(),
                lat,
                lon,
                epoch_secs: now,
            }
            .save(GPS_CACHE_FILE);
            return village.code;
        }

        eprintln!("{{\"text\":\"⛔️\", \"tooltip\":\"no village found for coordinates\"}}");
        exit(0)
    }

    if let Some(ref name) = args.name {
        let conn = wilayah::open().unwrap_or_else(|e| {
            eprintln!(
                "{{\"text\":\"⛔️\", \"tooltip\":\"failed to open location db: {}\"}}",
                e
            );
            exit(0)
        });

        let results = wilayah::find_by_name(&conn, name, 10).unwrap_or_else(|e| {
            eprintln!(
                "{{\"text\":\"⛔️\", \"tooltip\":\"name lookup failed: {}\"}}",
                e
            );
            exit(0)
        });

        if results.is_empty() {
            eprintln!(
                "{{\"text\":\"⛔️\", \"tooltip\":\"no village found matching '{}'\"}}",
                name
            );
            exit(0)
        }

        return results[0].code.clone();
    }

    eprintln!("{{\"text\":\"⛔️\", \"tooltip\":\"provide --adm4, --lat/--lon, or --name\"}}");
    exit(0)
}

fn main() {
    let args = Args::parse();
    let lang = args.lang;

    let adm4 = resolve_adm4(&args);

    let mut data = HashMap::new();

    let weather_url = format!(
        "https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4={}",
        adm4
    );
    let cachefile = format!("/tmp/cuaca-{}.json", adm4);

    let mut iterations = 0;
    let threshold = 20;

    let is_cache_file_recent = metadata(&cachefile).is_ok_and(|meta| {
        let ten_minutes_ago = SystemTime::now() - Duration::from_secs(600);
        meta.modified()
            .is_ok_and(|mod_time| mod_time > ten_minutes_ago)
    });

    let client = Client::new();
    let weather = if is_cache_file_recent {
        let json_str = read_to_string(&cachefile).unwrap();
        serde_json::from_str::<Value>(&json_str).unwrap()
    } else {
        loop {
            match client.get(&weather_url).send() {
                Ok(response) => match response.json::<Value>() {
                    Ok(json) => break json,
                    Err(_) => {
                        println!(
                            "{{\"text\":\"\u{26d3}\u{fe0f}\", \"tooltip\":\"invalid BMKG response\"}}"
                        );
                        exit(0)
                    }
                },
                Err(_) => {
                    iterations += 1;
                    thread::sleep(Duration::from_millis(500 * iterations));

                    if iterations == threshold {
                        println!(
                            "{{\"text\":\"\u{26d3}\u{fe0f}\", \"tooltip\":\"cannot access BMKG API\"}}"
                        );
                        exit(0)
                    }
                }
            }
        }
    };

    if !is_cache_file_recent {
        let mut file = File::create(&cachefile).unwrap_or_else(|_| {
            eprintln!("Unable to create cache file at {}", cachefile);
            exit(1);
        });
        file.write_all(serde_json::to_string_pretty(&weather).unwrap().as_bytes())
            .unwrap_or_else(|_| {
                eprintln!("Unable to write cache file at {}", cachefile);
                exit(1);
            });
    }

    let lokasi = &weather["lokasi"];
    let cuaca_groups = match weather["data"]
        .as_array()
        .and_then(|d| d.first())
        .and_then(|day| day["cuaca"].as_array())
    {
        Some(groups) => groups,
        None => {
            println!(
                "{{\"text\":\"\u{26d3}\u{fe0f}\", \"tooltip\":\"invalid BMKG data structure\"}}"
            );
            exit(0);
        }
    };

    let all_slots: Vec<&Value> = cuaca_groups
        .iter()
        .flat_map(|group| {
            group
                .as_array()
                .map_or(vec![], |slots| slots.iter().collect())
        })
        .collect();

    if all_slots.is_empty() {
        println!("{{\"text\":\"\u{26d3}\u{fe0f}\", \"tooltip\":\"no forecast data available\"}}");
        exit(0);
    }

    let first_slot = all_slots[0];
    let weather_code = first_slot["weather"].as_u64().unwrap_or(0) as u32;
    let weather_icon = get_weather_icon(weather_code, args.nerd);
    let weather_desc_key = lang.weather_desc_key();

    let text = match &args.custom_indicator {
        None => {
            let indicator = format_temp(first_slot["t"].as_i64().unwrap_or(0));

            format!("{} {}", weather_icon, indicator)
        }
        Some(expression) => format_indicator(first_slot, expression, weather_icon),
    };
    data.insert("text", text);

    let mut tooltip = format!(
        "<b>{}</b> {}\u{00b0}\n",
        first_slot[weather_desc_key].as_str().unwrap_or("?"),
        first_slot["t"].as_i64().unwrap_or(0),
    );

    tooltip += &format!(
        "{}: {}%\n",
        lang.humidity(),
        first_slot["hu"].as_i64().unwrap_or(0)
    );

    let tcc = first_slot["tcc"].as_i64().unwrap_or(0);
    tooltip += &format!("{}: {}%\n", lang.cloud_cover(), tcc);

    let tp = first_slot["tp"].as_f64().unwrap_or(0.0);
    tooltip += &format!("{}: {} mm\n", lang.precipitation(), tp);

    let ws = first_slot["ws"].as_f64().unwrap_or(0.0);
    let wd_deg = first_slot["wd_deg"].as_i64().unwrap_or(0);
    let wd_cardinal = first_slot["wd"].as_str().unwrap_or("?");
    tooltip += &format!(
        "{}: {} {} {} km/h\n",
        lang.wind(),
        format_wind_dir_icon(wd_deg, args.nerd),
        wd_cardinal,
        ws
    );

    let vs_text = first_slot["vs_text"].as_str().unwrap_or("?");
    tooltip += &format!("{}: {}\n", lang.visibility(), vs_text);

    let provinsi = lokasi["provinsi"].as_str().unwrap_or("");
    let kotkab = lokasi["kotkab"].as_str().unwrap_or("");
    let kecamatan = lokasi["kecamatan"].as_str().unwrap_or("");
    let desa = lokasi["desa"].as_str().unwrap_or("");

    let location_parts: Vec<&str> = vec![desa, kecamatan, kotkab, provinsi]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();

    tooltip += &format!("{}: {}\n", lang.location(), location_parts.join(", "));

    let locale = Locale::en_US;

    let mut all_flat_slots: Vec<(&Value, usize)> = cuaca_groups
        .iter()
        .enumerate()
        .flat_map(|(group_idx, group)| {
            group.as_array().map_or(vec![], |slots| {
                slots.iter().map(move |slot| (slot, group_idx)).collect()
            })
        })
        .collect();

    all_flat_slots.sort_by_key(|(slot, _)| {
        slot["local_datetime"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default()
    });

    let mut current_group: Option<usize> = None;
    for (slot, group_idx) in &all_flat_slots {
        if *group_idx != current_group.unwrap_or(999) {
            tooltip += "\n<b>";
            current_group = Some(*group_idx);

            let local_dt = slot["local_datetime"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let slot_date = NaiveDateTime::parse_from_str(&local_dt, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.date());

            if let Some(date) = slot_date {
                if *group_idx == 0 {
                    tooltip += &format!("{}, ", lang.today());
                } else if *group_idx == 1 {
                    tooltip += &format!("{}, ", lang.tomorrow());
                }

                tooltip += &format!(
                    "{}</b>\n",
                    date.format_localized(args.date_format.as_str(), locale)
                );
            }
        }

        let local_dt = slot["local_datetime"].as_str().unwrap_or("??:??");

        let desc = slot[weather_desc_key].as_str().unwrap_or("?");
        let ws_val = slot["ws"].as_f64().unwrap_or(0.0);
        let wd_val = slot["wd_deg"].as_i64().unwrap_or(0);

        let mut line = format!(
            "{}  {}  {}  {}  {} {} km/h",
            format_time(local_dt, args.ampm),
            get_weather_icon(slot["weather"].as_u64().unwrap_or(0) as u32, args.nerd),
            format_temp(slot["t"].as_i64().unwrap_or(0)),
            desc,
            format_wind_dir_icon(wd_val, args.nerd),
            ws_val,
        );

        if !args.hide_details {
            let tcc_val = slot["tcc"].as_i64().unwrap_or(0);
            let tp_val = slot["tp"].as_f64().unwrap_or(0.0);
            let vs_val = slot["vs_text"].as_str().unwrap_or("?");

            line += &format!(
                "  \u{2601}\u{fe0f} {}%  \u{1f327}\u{fe0f} {} mm  \u{1f441}\u{fe0f} {}",
                tcc_val, tp_val, vs_val
            );
        }

        line += "\n";
        tooltip += &line;
    }

    tooltip += &format!("\n<small>{}</small>", lang.source());

    data.insert("tooltip", tooltip);

    let css_class = first_slot[weather_desc_key]
        .as_str()
        .unwrap_or("unknown")
        .to_lowercase()
        .split(',')
        .next()
        .map(|s| s.trim().replace(' ', "_"))
        .unwrap_or_default();
    data.insert("class", css_class);

    let json_data = json!(data);
    println!("{}", json_data);
}
