use chrono::{Locale, NaiveDateTime};
use serde_json::Value;

use crate::constants::get_ascii_icon;
use crate::format::{celsius_to_fahrenheit, format_temp};
use crate::lang::Lang;

const VLINE: &str = "│";
const HLINE: &str = "─";
const TL: &str = "┌";
const TR: &str = "┐";
const BL: &str = "└";
const BR: &str = "┘";
const TOP_T: &str = "┬";
const BOTTOM_T: &str = "┴";

const CELL_WIDTH: usize = 29;
const ICON_WIDTH: usize = 13;
const DATA_WIDTH: usize = CELL_WIDTH - ICON_WIDTH;
const ICON_COLS: usize = 5;

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

    let humidity = first_slot["hu"].as_i64().unwrap_or(0);
    let ws = first_slot["ws"].as_f64().unwrap_or(0.0);
    let wd_deg = first_slot["wd_deg"].as_i64().unwrap_or(0);
    let wd_cardinal = first_slot["wd"].as_str().unwrap_or("?");
    let tcc = first_slot["tcc"].as_i64().unwrap_or(0);
    let tp = first_slot["tp"].as_f64().unwrap_or(0.0);
    let vs_text = first_slot["vs_text"].as_str().unwrap_or("?");

    let mut out = String::new();

    out.push_str(&format!("Weather report: {}\n", location));
    out.push('\n');

    out.push_str(&format!(
        "  {:<16} {}°C  {}: {}%\n",
        desc,
        temp,
        lang.humidity(),
        humidity
    ));
    out.push_str(&format!(
        "                  {}: {} {} {} km/h\n",
        lang.wind(),
        format_wind_dir_icon(wd_deg),
        wd_cardinal,
        ws
    ));
    out.push_str(&format!(
        "                  {}: {}%  {}: {}mm\n",
        lang.cloud_cover(),
        tcc,
        lang.precipitation(),
        tp
    ));
    out.push_str(&format!(
        "                  {}: {}\n",
        lang.visibility(),
        vs_text
    ));

    let locale = Locale::en_US;
    let day_labels = [lang.today(), lang.tomorrow(), lang.day_after_tomorrow()];

    for (group_idx, group) in cuaca_groups.iter().enumerate() {
        let slots: Vec<&Value> = group.as_array().map_or(vec![], |s| s.iter().collect());
        if slots.is_empty() || group_idx >= 3 {
            continue;
        }

        let ncols = slots.len();

        let date_str = slots
            .first()
            .and_then(|s| s["local_datetime"].as_str())
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S").ok())
            .map(|dt| dt.date().format_localized("%a %d %b", locale).to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let day_label = day_labels.get(group_idx).unwrap_or(&"");
        let header = if !day_label.is_empty() {
            format!("{}, {}", day_label, date_str)
        } else {
            date_str
        };

        let total_width = ncols * CELL_WIDTH + (ncols - 1);
        let header_len = header.len();
        let header_start = (total_width - header_len) / 2;
        let header_pad = format!("{:header_start$}", "");

        out.push('\n');
        out.push_str(&format!("{}{}\n", header_pad, header));

        out.push_str(TL);
        for i in 0..ncols {
            out.push_str(&HLINE.repeat(CELL_WIDTH));
            if i < ncols - 1 {
                out.push_str(TOP_T);
            } else {
                out.push_str(TR);
            }
        }
        out.push('\n');

        for row_idx in 0..ICON_COLS {
            out.push_str(VLINE);
            for slot in slots.iter().take(ncols) {
                let cell = render_cell_row(slot, row_idx, lang, fahrenheit);
                out.push_str(&format!("{:<CELL_WIDTH$}{}", cell, VLINE));
            }
            out.push('\n');
        }

        out.push_str(BL);
        for i in 0..ncols {
            out.push_str(&HLINE.repeat(CELL_WIDTH));
            if i < ncols - 1 {
                out.push_str(BOTTOM_T);
            } else {
                out.push_str(BR);
            }
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&format!("{}\n", lang.source()));

    out
}

fn render_cell_row(slot: &Value, row: usize, lang: Lang, fahrenheit: bool) -> String {
    let weather_desc_key = lang.weather_desc_key();
    let icon = get_ascii_icon(slot["weather"].as_u64().unwrap_or(0) as u32);
    let icon_line = icon[row];

    let data = get_cell_data(slot, row, weather_desc_key, fahrenheit);

    format!("{:<ICON_WIDTH$}{:<DATA_WIDTH$}", icon_line, data)
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
