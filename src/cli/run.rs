//! Orchestration of the application logic.

use super::args::Args;
use crate::color;
use crate::constants::{
    get_ascii_icon, CLOUD_COVER_ICON, ERROR_ICON, PRECIPITATION_ICON, VISIBILITY_ICON,
};
use crate::core::cache::cache_dir;
use crate::core::error::CuacaError;
use crate::core::location;
use crate::core::warnings;
use crate::core::weather;
use crate::format::{
    self, format_indicator, format_temp, format_time, format_wind_dir_icon, get_weather_icon,
};
use crate::lang::Lang;
use crate::terminal::render_terminal;
use crate::util;
use chrono::{DateTime, Locale, NaiveDateTime, Utc};
use reqwest::blocking::Client;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::fs::metadata;
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};

use crate::cli::args::OutputFormat;

/// Main entry point for the CLI application.
///
/// Parses arguments, sets color mode, resolves location, fetches weather,
/// and renders output according to the selected format.
pub fn run(args: Args) -> Result<(), CuacaError> {
    // set color mode
    color::set_color_mode(args.color);
    let lang = args.lang;

    // Resolve location
    let adm4 = location::resolve(
        args.adm4.as_deref(),
        args.lat,
        args.lon,
        args.name.as_deref(),
    )?;

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
                        util::error_json(ERROR_ICON, "corrupted cache, fetching fresh data")
                    );
                    weather::fetch_weather(
                        &client,
                        &weather_url,
                        &cachefile,
                        &mut iterations,
                        threshold,
                    )?
                }
            },
            Err(_) => {
                println!(
                    "{}",
                    util::error_json(ERROR_ICON, "cache read error, fetching fresh data")
                );
                weather::fetch_weather(
                    &client,
                    &weather_url,
                    &cachefile,
                    &mut iterations,
                    threshold,
                )?
            }
        }
    } else {
        weather::fetch_weather(
            &client,
            &weather_url,
            &cachefile,
            &mut iterations,
            threshold,
        )?
    };

    // Parse weather JSON structure
    let lokasi = &weather["lokasi"];
    let cuaca_groups = match weather["data"]
        .as_array()
        .and_then(|d| d.first())
        .and_then(|day| day["cuaca"].as_array())
    {
        Some(groups) => groups,
        None => return Err(CuacaError::Data("invalid BMKG data structure".to_string())),
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
        return Err(CuacaError::Data("no forecast data available".to_string()));
    }

    // Fetch warnings if requested
    let warnings_list = if args.warnings {
        let province = weather["lokasi"]["provinsi"].as_str().unwrap_or("");
        let lat = weather["lokasi"]["lat"].as_f64().unwrap_or(0.0);
        let lon = weather["lokasi"]["lon"].as_f64().unwrap_or(0.0);
        warnings::fetch_warnings(province, args.lang, lat, lon, args.warnings_ttl)
            .unwrap_or_default()
    } else {
        vec![]
    };

    let first_slot = all_slots[0];
    let weather_code = first_slot["weather"].as_u64().unwrap_or(0) as u32;
    let weather_icon = get_weather_icon(weather_code, args.nerd);
    let weather_desc_key = lang.weather_desc_key();

    let temp_c = first_slot["t"].as_i64().unwrap_or(0);
    let display_temp = if args.fahrenheit {
        format::celsius_to_fahrenheit(temp_c)
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
        format::celsius_to_fahrenheit(first_temp)
    } else {
        first_temp
    };

    let first_desc = first_slot[weather_desc_key].as_str().unwrap_or("?");
    let escaped_first_desc = util::escape_pango(first_desc);

    // Compute location parts
    let provinsi = lokasi["provinsi"].as_str().unwrap_or("");
    let kotkab = lokasi["kotkab"].as_str().unwrap_or("");
    let kecamatan = lokasi["kecamatan"].as_str().unwrap_or("");
    let desa = lokasi["desa"].as_str().unwrap_or("");
    let location_parts: Vec<&str> = vec![desa, kecamatan, kotkab, provinsi]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();

    // Build tooltip header
    let mut tooltip = String::new();
    tooltip.push_str(&format!(
        "<b>{}:</b> {}\n",
        lang.weather_report(),
        util::escape_pango(&location_parts.join(", "))
    ));
    tooltip.push('\n'); // blank line after title
    tooltip.push_str(&format!(
        "<b>{}</b> {} {}\n",
        escaped_first_desc, display_temp, unit
    ));

    // Build ASCII art column (5 lines)
    let raw_ascii = get_ascii_icon(weather_code);
    let ascii_lines: Vec<String> = raw_ascii
        .iter()
        .map(|&line| color::expand_color_tokens(line))
        .map(|ansi_line| color::ansi_to_pango(&ansi_line))
        .collect();

    // Prepare detail lines without description
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
    let vs_detail = format!("{}: {}", lang.visibility(), util::escape_pango(vs_text));

    let detail_lines = [
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
            format::celsius_to_fahrenheit(temp_c)
        } else {
            temp_c
        };

        let mut line = format!(
            "{}  {}  {}  {}  {} {} {}",
            format_time(local_dt, args.ampm),
            get_weather_icon(slot["weather"].as_u64().unwrap_or(0) as u32, args.nerd),
            format_temp(temp),
            util::escape_pango(desc),
            format_wind_dir_icon(wd_val, args.nerd),
            ws_val,
            lang.wind_unit(),
        );

        if !args.hide_details {
            let tcc_val = slot["tcc"].as_i64().unwrap_or(0);
            let tp_val = slot["tp"].as_f64().unwrap_or(0.0);
            let vs_val = slot["vs_text"].as_str().unwrap_or("?");

            let (cloud_icon, rain_icon, eye_icon) = if args.nerd {
                ("\u{f0330}", "\u{f0317}", "\u{f02fd}")
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
                util::escape_pango(vs_val)
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
            tooltip += &format!("• <b>{}<br/>{}", util::escape_pango(&w.headline), times);
            if let Some(web) = &w.web {
                tooltip += &format!("<a href=\"{}\">Infographic</a>", util::escape_pango(web));
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
            let json_data = serde_json::json!(data);
            println!("{}", json_data);
        }
        OutputFormat::Text => {
            let output = render_terminal(&weather, &args, &warnings_list);
            println!("{}", output);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&weather).unwrap());
        }
    }

    Ok(())
}
