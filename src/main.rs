use std::collections::HashMap;
use std::fs::{create_dir_all, metadata, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Locale, NaiveDateTime, Utc};
use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use cuaca::cache::cache_dir;
use cuaca::cli::{Args, OutputFormat};
use cuaca::color;
use cuaca::constants::{
    get_ascii_icon, CLOUD_COVER_ICON, ERROR_ICON, PRECIPITATION_ICON, VISIBILITY_ICON,
};
use cuaca::format::{
    celsius_to_fahrenheit, format_indicator, format_temp, format_time, format_wind_dir_icon,
    get_weather_icon,
};
use cuaca::lang::Lang;
use cuaca::terminal;
use cuaca::util::escape_pango;
use cuaca::warnings;

fn error_json(text: &str, tooltip: &str) -> String {
    format!("{{\"text\":\"{}\", \"tooltip\":\"{}\"}}", text, tooltip)
}

#[derive(Deserialize)]
struct GpsCache {
    adm4: String,
    lat: f64,
    lon: f64,
    epoch_secs: u64,
}

impl GpsCache {
    fn save(&self, path: &PathBuf) {
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
    const GPS_CACHE_MAX_AGE_SECS: u64 = 86400;

    if let Some(ref code) = args.adm4 {
        return code.clone();
    }

    if args.lat.is_some() && args.lon.is_none() {
        println!("{}", error_json("⛔️", "--lat requires --lon"));
        exit(1);
    }
    if args.lon.is_some() && args.lat.is_none() {
        println!("{}", error_json("⛔️", "--lon requires --lat"));
        exit(1);
    }

    if let (Some(lat), Some(lon)) = (args.lat, args.lon) {
        let gps_cache_file = cache_dir().join("cuaca-gps.json");
        let cache_valid = read_to_string(&gps_cache_file)
            .ok()
            .and_then(|s| serde_json::from_str::<GpsCache>(&s).ok())
            .and_then(|c| {
                if c.lat == lat && c.lon == lon && !c.is_stale(GPS_CACHE_MAX_AGE_SECS) {
                    Some(c.adm4)
                } else {
                    None
                }
            });

        if let Some(adm4) = cache_valid {
            return adm4;
        }

        let conn = wilayah::open().unwrap_or_else(|e| {
            println!(
                "{}",
                error_json("⛔️", &format!("failed to open location db: {}", e))
            );
            exit(1)
        });

        let results = wilayah::find_nearest(&conn, lat, lon, 1).unwrap_or_else(|e| {
            println!(
                "{}",
                error_json("⛔️", &format!("location lookup failed: {}", e))
            );
            exit(1)
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
            .save(&gps_cache_file);
            return village.code;
        }

        println!("{}", error_json("⛔️", "no village found for coordinates"));
        exit(1)
    }

    if let Some(ref name) = args.name {
        let conn = wilayah::open().unwrap_or_else(|e| {
            println!(
                "{}",
                error_json("⛔️", &format!("failed to open location db: {}", e))
            );
            exit(1)
        });

        let results = wilayah::find_by_name(&conn, name, 10).unwrap_or_else(|e| {
            println!(
                "{}",
                error_json("⛔️", &format!("name lookup failed: {}", e))
            );
            exit(1)
        });

        if results.is_empty() {
            println!(
                "{}",
                error_json("⛔️", &format!("no village found matching '{}'", name))
            );
            exit(1)
        }

        return results[0].code.clone();
    }

    println!(
        "{}",
        error_json("⛔️", "provide --adm4, --lat/--lon, or --name")
    );
    exit(1)
}

fn main() {
    let args = Args::parse();
    color::set_color_mode(args.color);
    let lang = args.lang;

    let adm4 = resolve_adm4(&args);

    let mut data = HashMap::new();

    let weather_url = format!(
        "https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4={}",
        adm4
    );
    let cachefile = cache_dir().join(format!("cuaca-{}.json", adm4));

    let mut iterations = 0;
    let threshold = 20;

    let is_cache_file_recent = metadata(&cachefile).is_ok_and(|meta| {
        let ten_minutes_ago = SystemTime::now() - Duration::from_secs(600);
        meta.modified()
            .is_ok_and(|mod_time| mod_time > ten_minutes_ago)
    });

    let client = Client::new();
    let weather = if is_cache_file_recent {
        match read_to_string(&cachefile) {
            Ok(json_str) => match serde_json::from_str::<Value>(&json_str) {
                Ok(json) => json,
                Err(_) => {
                    println!(
                        "{}",
                        error_json(ERROR_ICON, "corrupted cache, fetching fresh data")
                    );
                    fetch_weather(
                        &client,
                        &weather_url,
                        &cachefile,
                        &mut iterations,
                        threshold,
                    )
                }
            },
            Err(_) => {
                println!(
                    "{}",
                    error_json(ERROR_ICON, "cache read error, fetching fresh data")
                );
                fetch_weather(
                    &client,
                    &weather_url,
                    &cachefile,
                    &mut iterations,
                    threshold,
                )
            }
        }
    } else {
        fetch_weather(
            &client,
            &weather_url,
            &cachefile,
            &mut iterations,
            threshold,
        )
    };

    let lokasi = &weather["lokasi"];
    let cuaca_groups = match weather["data"]
        .as_array()
        .and_then(|d| d.first())
        .and_then(|day| day["cuaca"].as_array())
    {
        Some(groups) => groups,
        None => {
            println!("{}", error_json(ERROR_ICON, "invalid BMKG data structure"));
            exit(1);
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
        println!("{}", error_json(ERROR_ICON, "no forecast data available"));
        exit(1);
    }

    // Fetch warnings if requested
    let warnings_list = if args.warnings {
        let province = weather["lokasi"]["provinsi"].as_str().unwrap_or("");
        let lat = weather["lokasi"]["lat"].as_f64().unwrap_or(0.0);
        let lon = weather["lokasi"]["lon"].as_f64().unwrap_or(0.0);
        warnings::fetch_warnings(province, args.lang, lat, lon, args.warnings_ttl)
    } else {
        vec![]
    };

    let first_slot = all_slots[0];
    let weather_code = first_slot["weather"].as_u64().unwrap_or(0) as u32;
    let weather_icon = get_weather_icon(weather_code, args.nerd);
    let weather_desc_key = lang.weather_desc_key();

    let temp_c = first_slot["t"].as_i64().unwrap_or(0);
    let display_temp = if args.fahrenheit {
        celsius_to_fahrenheit(temp_c)
    } else {
        temp_c
    };
    let unit = if args.fahrenheit { "°F" } else { "°C" };

    let mut bar_text = match &args.custom_indicator {
        None => {
            let indicator = format_temp(display_temp);
            format!("{} {}", weather_icon, indicator)
        }
        Some(expression) => format_indicator(first_slot, expression, weather_icon),
    };
    if args.warnings && !warnings_list.is_empty() {
        bar_text = format!("⚠️ {}", bar_text);
    }
    data.insert("text", bar_text);

    let first_temp = first_slot["t"].as_i64().unwrap_or(0);
    let display_temp = if args.fahrenheit {
        celsius_to_fahrenheit(first_temp)
    } else {
        first_temp
    };

    let first_desc = first_slot[weather_desc_key].as_str().unwrap_or("?");
    let escaped_first_desc = escape_pango(first_desc);

    // Compute location parts early
    let provinsi = lokasi["provinsi"].as_str().unwrap_or("");
    let kotkab = lokasi["kotkab"].as_str().unwrap_or("");
    let kecamatan = lokasi["kecamatan"].as_str().unwrap_or("");
    let desa = lokasi["desa"].as_str().unwrap_or("");
    let location_parts: Vec<&str> = vec![desa, kecamatan, kotkab, provinsi]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();

    // Build tooltip header: location first, separated by blank line, then description + temp + unit
    let mut tooltip = String::new();
    tooltip.push_str(&format!(
        "<b>{}:</b> {}\n",
        lang.weather_report(),
        escape_pango(&location_parts.join(", "))
    ));
    tooltip.push('\n'); // blank line after title
                        // Build ASCII art column (6 lines)
    let raw_ascii = get_ascii_icon(weather_code);
    let mut ascii_lines: Vec<String> = raw_ascii
        .iter()
        .map(|&line| color::ansi_to_pango(line))
        .collect();
    while ascii_lines.len() < 6 {
        ascii_lines.push(String::new());
    }

    // Prepare detail lines with proper escaping
    let desc_detail = format!("<b>{}</b> {} {}", escaped_first_desc, display_temp, unit);
    let hu_detail = format!(
        "{}: {}%",
        lang.humidity(),
        first_slot["hu"].as_i64().unwrap_or(0)
    );
    let tcc = first_slot["tcc"].as_i64().unwrap_or(0);
    let tcc_detail = format!("{}: {}%", lang.cloud_cover(), tcc);
    let tp = first_slot["tp"].as_f64().unwrap_or(0.0);
    let tp_detail = format!("{}: {:.1} mm", lang.precipitation(), tp);
    let ws = first_slot["ws"].as_f64().unwrap_or(0.0);
    let wd_deg = first_slot["wd_deg"].as_i64().unwrap_or(0);
    let wd_cardinal = first_slot["wd"].as_str().unwrap_or("?");
    let wind_detail = format!(
        "{}: {} {} {} {}",
        lang.wind(),
        format_wind_dir_icon(wd_deg, args.nerd),
        wd_cardinal,
        ws,
        lang.wind_unit()
    );
    let vs_text = first_slot["vs_text"].as_str().unwrap_or("?");
    let vs_detail = format!("{}: {}", lang.visibility(), escape_pango(vs_text));

    let detail_lines = [
        &desc_detail,
        &hu_detail,
        &tcc_detail,
        &tp_detail,
        &wind_detail,
        &vs_detail,
    ];

    for (art, detail) in ascii_lines.iter().zip(detail_lines.iter()) {
        let art_mono = if art.is_empty() {
            String::new()
        } else {
            format!(r#"<span font_family="monospace">{}</span>"#, art)
        };
        tooltip.push_str(&format!("{} {}\n", art_mono, detail));
    }

    // Blank line before warnings/day sections
    tooltip.push('\n');

    let locale = match lang {
        Lang::EN => Locale::en_US,
        Lang::ID => Locale::id_ID,
    };

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
        if current_group.is_none() || current_group != Some(*group_idx) {
            if *group_idx == 0 {
                tooltip += "<b>";
            } else {
                tooltip += "\n<b>";
            }
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
                } else if *group_idx == 2 {
                    tooltip += &format!("{}, ", lang.day_after_tomorrow());
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

        let temp_c = slot["t"].as_i64().unwrap_or(0);
        let temp = if args.fahrenheit {
            celsius_to_fahrenheit(temp_c)
        } else {
            temp_c
        };

        let mut line = format!(
            "{}  {}  {}  {}  {} {} {}",
            format_time(local_dt, args.ampm),
            get_weather_icon(slot["weather"].as_u64().unwrap_or(0) as u32, args.nerd),
            format_temp(temp),
            escape_pango(desc),
            format_wind_dir_icon(wd_val, args.nerd),
            ws_val,
            lang.wind_unit(),
        );

        if !args.hide_details {
            let tcc_val = slot["tcc"].as_i64().unwrap_or(0);
            let tp_val = slot["tp"].as_f64().unwrap_or(0.0);
            let vs_val = slot["vs_text"].as_str().unwrap_or("?");

            let (cloud_icon, rain_icon, eye_icon) = if args.nerd {
                ("\u{F0330}", "\u{F0317}", "\u{F02FD}")
            } else {
                (CLOUD_COVER_ICON, PRECIPITATION_ICON, VISIBILITY_ICON)
            };

            line += &format!(
                "  {} {}%  {} {} mm  {} {}",
                cloud_icon,
                tcc_val,
                rain_icon,
                tp_val,
                eye_icon,
                escape_pango(vs_val)
            );
        }

        line += "\n";
        tooltip += &line;
    }

    // Append weather warnings if any
    if !warnings_list.is_empty() {
        tooltip += "<b>Weather Warnings:</b>\n";
        for w in &warnings_list {
            // Validity times: HH:MM–HH:MM (24‑hour, local)
            let times = if let (Some(e), Some(x)) = (&w.effective, (&w.expires)) {
                let fmt = |dt: &DateTime<Utc>| dt.format("%H:%M").to_string();
                format!("Valid: {}–{}\n", fmt(e), fmt(x))
            } else {
                "".to_string()
            };
            tooltip += &format!("• <b>{}<br/>{}", escape_pango(&w.headline), times);
            if let Some(web) = &w.web {
                tooltip += &format!("<a href=\"{}\">Infographic</a>", escape_pango(web));
            }
            tooltip += "</br>\n";
        }
        tooltip += "\n";
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

    match args.format {
        OutputFormat::Bar => {
            let json_data = json!(data);
            println!("{}", json_data);
        }
        OutputFormat::Text => {
            let output = terminal::render_terminal(&weather, &args, &warnings_list);
            println!("{}", output);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&weather).unwrap());
        }
    }
}

fn fetch_weather(
    client: &Client,
    weather_url: &str,
    cachefile: &PathBuf,
    iterations: &mut usize,
    threshold: usize,
) -> Value {
    loop {
        match client.get(weather_url).send() {
            Ok(response) => match response.json::<Value>() {
                Ok(json) => {
                    save_cache(cachefile, &json);
                    break json;
                }
                Err(_) => {
                    println!("{}", error_json(ERROR_ICON, "invalid BMKG response"));
                    exit(1)
                }
            },
            Err(_) => {
                *iterations += 1;
                thread::sleep(Duration::from_millis(500 * *iterations as u64));

                if *iterations == threshold {
                    println!("{}", error_json(ERROR_ICON, "cannot access BMKG API"));
                    exit(1)
                }
            }
        }
    }
}

fn save_cache(cachefile: &PathBuf, weather: &Value) {
    if let Some(parent) = cachefile.parent() {
        create_dir_all(parent).unwrap_or_else(|_| {
            eprintln!("Unable to create cache directory at {}", parent.display());
            exit(1);
        });
    }
    let mut file = File::create(cachefile).unwrap_or_else(|_| {
        eprintln!("Unable to create cache file at {}", cachefile.display());
        exit(1);
    });
    file.write_all(serde_json::to_string_pretty(&weather).unwrap().as_bytes())
        .unwrap_or_else(|_| {
            eprintln!("Unable to write cache file at {}", cachefile.display());
            exit(1);
        });
}
