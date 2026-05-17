use chrono::{Locale, NaiveDateTime};
use serde_json::Value;

use crate::format::{celsius_to_fahrenheit, format_temp, format_wind_dir_icon};
use crate::lang::Lang;

const VLINE: &str = "│";
const TL: &str = "┌";
const TR: &str = "┐";
const BL: &str = "└";
const BR: &str = "┘";
const CROSS: &str = "┼";
const TOP_T: &str = "┬";
const BOTTOM_T: &str = "┴";
const LEFT_T: &str = "├";
const RIGHT_T: &str = "┤";

const CELL_WIDTH: usize = 30;
const COLS: usize = 3;
const TAB_WIDTH: usize = 13;

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
        format_wind_dir_icon(wd_deg, args.nerd),
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
    let time_labels = ["Morning", "Noon", "Evening"];

    for (group_idx, group) in cuaca_groups.iter().enumerate() {
        let slots: Vec<&Value> = group.as_array().map_or(vec![], |s| s.iter().collect());
        if slots.is_empty() || group_idx >= 3 {
            continue;
        }

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

        out.push('\n');
        out.push_str(&format!("{:TAB_WIDTH$}{} {}\n", "", TL, &header,));
        out.push_str(&format!("{:TAB_WIDTH$}{}\n", "", TR,));

        out.push_str(&format!("{:TAB_WIDTH$}{}", "", TL));
        for i in 0..COLS {
            let label = time_labels.get(i).unwrap_or(&"");
            out.push_str(&format!(
                "{:^CELL_WIDTH$}{}",
                label,
                if i < COLS - 1 { TOP_T } else { TR }
            ));
        }
        out.push('\n');

        out.push_str(&format!("{:TAB_WIDTH$}{}", "", LEFT_T));
        for i in 0..COLS {
            out.push_str(&format!("{:CELL_WIDTH$}", ""));
            if i < COLS - 1 {
                out.push_str(CROSS);
            } else {
                out.push_str(RIGHT_T);
            }
        }
        out.push('\n');

        for row_idx in 0..5 {
            out.push_str(&format!("{:TAB_WIDTH$}{}", "", VLINE));
            for slot in slots.iter().take(COLS) {
                let cell = render_cell_row(slot, row_idx, lang, fahrenheit, args.nerd);
                out.push_str(&format!("{:<CELL_WIDTH$}{}", cell, VLINE));
            }
            out.push('\n');
        }

        out.push_str(&format!("{:TAB_WIDTH$}{}", "", BL));
        for i in 0..COLS {
            out.push_str(&format!("{:CELL_WIDTH$}", ""));
            if i < COLS - 1 {
                out.push_str(BOTTOM_T);
            } else {
                out.push_str(BR);
            }
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&format!("Source: {}", lang.source()));

    out
}

fn render_cell_row(slot: &Value, row: usize, lang: Lang, fahrenheit: bool, nerd: bool) -> String {
    let weather_desc_key = lang.weather_desc_key();
    let ws = slot["ws"].as_f64().unwrap_or(0.0);
    let wd_deg = slot["wd_deg"].as_i64().unwrap_or(0);
    let wd_cardinal = slot["wd"].as_str().unwrap_or("?");
    let temp_c = slot["t"].as_i64().unwrap_or(0);
    let temp = if fahrenheit {
        celsius_to_fahrenheit(temp_c)
    } else {
        temp_c
    };

    let icon = get_ascii_icon(slot["weather"].as_u64().unwrap_or(0) as u32);

    match row {
        0 => format!("{:>16}", icon.0),
        1 => format!("{:>16}", icon.1),
        2 => {
            let desc = slot[weather_desc_key].as_str().unwrap_or("?");
            if desc.len() > CELL_WIDTH {
                desc[..CELL_WIDTH].to_string()
            } else {
                desc.to_string()
            }
        }
        3 => format!("  {:>10}°C", format_temp(temp)),
        4 => {
            let dir_icon = format_wind_dir_icon(wd_deg, nerd);
            format!("  {} {} {} km/h", dir_icon, wd_cardinal, ws)
        }
        _ => String::new(),
    }
}

fn get_ascii_icon(code: u32) -> (&'static str, &'static str) {
    match code {
        0 | 1 => ("   \\  /     ", " _ / \"\".-.  "),
        2 => ("    \\   /   ", "     .-.     "),
        3 => ("             ", " .--.        "),
        4 => ("             ", " .--.        "),
        5 | 10 => ("             ", " - - - -     "),
        45 => ("             ", " - - - -     "),
        60 | 61 => (" _`/\"\".-.   ", "  ,\\_(   ).  "),
        63 => (" _`/\"\".-.   ", "  ,\\_(   ).  "),
        80 => (" _`/\"\".-.   ", "  ,\\_(   ).  "),
        95 | 97 => ("     .-.     ", "   (   ).    "),
        _ => ("             ", "     .-.     "),
    }
}
