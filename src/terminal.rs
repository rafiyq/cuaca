use std::collections::{BTreeSet, HashMap};

use chrono::{Locale, NaiveDateTime, Timelike};
use serde_json::Value;

use crate::constants::get_ascii_icon;
use crate::format::{celsius_to_fahrenheit, format_temp};

const VLINE: &str = "│";
const HLINE: &str = "─";
const TL: &str = "┌";
const BL: &str = "└";
const BR: &str = "┘";
const LEFT_T: &str = "├";
const RIGHT_T: &str = "┤";

const HOUR_COL_WIDTH: usize = 6;
const DAY_COL_WIDTH: usize = 30;
const ICON_WIDTH: usize = 13;
const DATA_WIDTH: usize = DAY_COL_WIDTH - ICON_WIDTH;
const ICON_ROWS: usize = 5;

pub fn render_terminal(weather: &Value, args: &crate::cli::Args) -> String {
    let lang = args.lang;
    let fahrenheit = args.fahrenheit;

    let lokasi = &weather["lokasi"];
    let provinsi = lokasi["provinsi"].as_str().unwrap_or("");
    let kotkab = lokasi["kotkab"].as_str().unwrap_or("");
    let location = if !kotkab.is_empty() {
        format!("{}, {}", kotkab, provinsi)
    } else {
        provinsi.to_string()
    };

    let cuaca_groups = match weather["data"]
        .as_array()
        .and_then(|d| d.first())
        .and_then(|day| day["cuaca"].as_array())
    {
        Some(g) => g,
        None => return "No forecast data available".to_string(),
    };

    let all_slots: Vec<&Value> = cuaca_groups
        .iter()
        .flat_map(|g| g.as_array().map_or(vec![], |s| s.iter().collect()))
        .collect();

    if all_slots.is_empty() {
        return "No forecast data available".to_string();
    }

    let first_slot = all_slots[0];
    let weather_desc_key = lang.weather_desc_key();
    let desc = first_slot[weather_desc_key].as_str().unwrap_or("?");
    let temp_c = first_slot["t"].as_i64().unwrap_or(0);
    let temp = if fahrenheit {
        celsius_to_fahrenheit(temp_c)
    } else {
        temp_c
    };

    let ws = first_slot["ws"].as_f64().unwrap_or(0.0);
    let wd_deg = first_slot["wd_deg"].as_i64().unwrap_or(0);
    let wd_cardinal = first_slot["wd"].as_str().unwrap_or("?");
    let tp = first_slot["tp"].as_f64().unwrap_or(0.0);
    let vs_text = first_slot["vs_text"].as_str().unwrap_or("?");

    let mut out = String::new();

    let first_code = first_slot["weather"].as_u64().unwrap_or(0) as u32;
    let icon = get_ascii_icon(first_code);
    let feels_like_c = first_slot["t"].as_i64().unwrap_or(0);
    let feels_like = if fahrenheit {
        celsius_to_fahrenheit(feels_like_c)
    } else {
        feels_like_c
    };

    out.push_str(&format!("Weather report: {}\n", location));
    out.push('\n');

    out.push_str(&format!("                {}\n", desc));
    out.push_str(&format!("{}  +{}({}) °C\n", icon[0], temp, feels_like));
    out.push_str(&format!(
        "{}  {} {} {} km/h\n",
        icon[1],
        format_wind_dir_icon(wd_deg),
        wd_cardinal,
        ws as i64
    ));
    out.push_str(&format!("{}  {}\n", icon[2], vs_text));
    out.push_str(&format!("                {} mm\n", tp));

    let locale = Locale::en_US;
    let day_labels = [lang.today(), lang.tomorrow(), lang.day_after_tomorrow()];

    let mut all_hours: BTreeSet<String> = BTreeSet::new();
    let mut day_data: Vec<HashMap<String, &Value>> = Vec::new();

    for group in cuaca_groups.iter() {
        let slots: Vec<&Value> = group.as_array().map_or(vec![], |s| s.iter().collect());
        let mut day_map = HashMap::new();
        for slot in slots {
            let hour = extract_hour(slot["local_datetime"].as_str());
            if let Some(ref h) = hour {
                all_hours.insert(h.clone());
                day_map.insert(h.clone(), slot);
            }
        }
        day_data.push(day_map);
    }

    let day_headers: Vec<String> = cuaca_groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let slots: Vec<&Value> = g.as_array().map_or(vec![], |s| s.iter().collect());
            let date_str = slots
                .first()
                .and_then(|s| s["local_datetime"].as_str())
                .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S").ok())
                .map(|dt| dt.date().format_localized("%a %d %b", locale).to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let label = day_labels.get(i).unwrap_or(&"");
            if !label.is_empty() {
                format!("{}, {}", label, date_str)
            } else {
                date_str
            }
        })
        .collect();

    let num_days = day_headers.len();

    out.push('\n');
    out.push_str(TL);
    out.push_str(&" ".repeat(HOUR_COL_WIDTH));
    for header in &day_headers {
        out.push_str(VLINE);
        out.push_str(&format!("{:^DAY_COL_WIDTH$}", header));
    }
    out.push_str(VLINE);
    out.push('\n');

    out.push_str(LEFT_T);
    out.push_str(&HLINE.repeat(HOUR_COL_WIDTH));
    for _ in 0..num_days {
        out.push_str(VLINE);
        out.push_str(&HLINE.repeat(DAY_COL_WIDTH));
    }
    out.push_str(RIGHT_T);
    out.push('\n');

    let hours: Vec<&String> = all_hours.iter().collect();
    for (hour_idx, hour) in hours.iter().enumerate() {
        for row_idx in 0..ICON_ROWS {
            if row_idx == 0 {
                out.push_str(VLINE);
                out.push_str(&format!("{:^HOUR_COL_WIDTH$}", hour));
            } else {
                out.push_str(VLINE);
                out.push_str(&" ".repeat(HOUR_COL_WIDTH));
            }
            for day_idx in 0..num_days {
                out.push_str(VLINE);
                if let Some(slot) = day_data.get(day_idx).and_then(|d| d.get(*hour)) {
                    let icon = get_ascii_icon(slot["weather"].as_u64().unwrap_or(0) as u32);
                    let icon_line = icon[row_idx];
                    let data = get_cell_data(slot, row_idx, lang.weather_desc_key(), fahrenheit);
                    out.push_str(&format!("{:<ICON_WIDTH$}{:<DATA_WIDTH$}", icon_line, data));
                } else {
                    out.push_str(&" ".repeat(DAY_COL_WIDTH));
                }
            }
            out.push_str(VLINE);
            out.push('\n');
        }

        if hour_idx < hours.len() - 1 {
            out.push_str(LEFT_T);
            out.push_str(&HLINE.repeat(HOUR_COL_WIDTH));
            for _ in 0..num_days {
                out.push_str(VLINE);
                out.push_str(&HLINE.repeat(DAY_COL_WIDTH));
            }
            out.push_str(RIGHT_T);
            out.push('\n');
        }
    }

    out.push_str(BL);
    out.push_str(&HLINE.repeat(HOUR_COL_WIDTH));
    for _ in 0..num_days {
        out.push_str(VLINE);
        out.push_str(&HLINE.repeat(DAY_COL_WIDTH));
    }
    out.push_str(BR);
    out.push('\n');

    out.push('\n');
    out.push_str(&format!("{}\n", lang.source()));

    out
}

fn get_cell_data(slot: &Value, row: usize, desc_key: &str, fahrenheit: bool) -> String {
    match row {
        0 => {
            let desc = slot[desc_key].as_str().unwrap_or("?");
            let max = DATA_WIDTH.saturating_sub(1);
            if desc.len() > max {
                format!("{}…", &desc[..max.saturating_sub(1)])
            } else {
                desc.to_string()
            }
        }
        1 => {
            let temp_c = slot["t"].as_i64().unwrap_or(0);
            let temp = if fahrenheit {
                celsius_to_fahrenheit(temp_c)
            } else {
                temp_c
            };
            format_temp(temp)
        }
        2 => {
            let wd_deg = slot["wd_deg"].as_i64().unwrap_or(0);
            let wd_cardinal = slot["wd"].as_str().unwrap_or("?");
            let ws = slot["ws"].as_f64().unwrap_or(0.0);
            format!(
                "{} {} {} km/h",
                format_wind_dir_icon(wd_deg),
                wd_cardinal,
                ws as i64
            )
        }
        3 => {
            let vs = slot["vs_text"].as_str().unwrap_or("?");
            vs.to_string()
        }
        4 => {
            let tp = slot["tp"].as_f64().unwrap_or(0.0);
            let hu = slot["hu"].as_i64().unwrap_or(0);
            format!("{:.1}mm | {}%", tp, hu)
        }
        _ => String::new(),
    }
}

fn extract_hour(local_datetime: Option<&str>) -> Option<String> {
    local_datetime
        .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| format!("{:02}:{:02}", dt.hour(), dt.minute()))
}

fn format_wind_dir_icon(degrees: i64) -> &'static str {
    let dir = ((degrees % 360) as f64 / 45.0).round() as usize % 8;
    [
        "\u{2b06}\u{fe0f}",
        "\u{2197}\u{fe0f}",
        "\u{27a1}\u{fe0f}",
        "\u{2198}\u{fe0f}",
        "\u{2b07}\u{fe0f}",
        "\u{2199}\u{fe0f}",
        "\u{2b05}\u{fe0f}",
        "\u{2196}\u{fe0f}",
    ][dir]
}
