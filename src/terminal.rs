use chrono::{Locale, NaiveDateTime, Timelike};
use serde_json::Value;

use crate::color;
use crate::constants::get_ascii_icon;
use crate::format::celsius_to_fahrenheit;
use crate::graphs::{sparkline_panel, temperature_panel};

pub fn render_terminal(weather: &Value, args: &crate::cli::Args) -> String {
    let lang = args.lang;
    let fahrenheit = args.fahrenheit;

    let lokasi = &weather["lokasi"];
    let provinsi = lokasi["provinsi"].as_str().unwrap_or("");
    let kotkab = lokasi["kotkab"].as_str().unwrap_or("");
    let kecamatan = lokasi["kecamatan"].as_str().unwrap_or("");
    let desa = lokasi["desa"].as_str().unwrap_or("");

    let location_parts: Vec<&str> = vec![desa, kecamatan, kotkab, provinsi]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();
    let location = location_parts.join(", ");

    let cuaca_groups = match weather["data"]
        .as_array()
        .and_then(|d| d.first())
        .and_then(|day| day["cuaca"].as_array())
    {
        Some(g) => g,
        None => return "No forecast data available".to_string(),
    };

    let all_slots: Vec<(usize, &Value)> = cuaca_groups
        .iter()
        .enumerate()
        .flat_map(|(g_idx, g)| {
            g.as_array().map_or(vec![], |s| {
                s.iter().map(move |slot| (g_idx, slot)).collect()
            })
        })
        .collect();

    if all_slots.is_empty() {
        return "No forecast data available".to_string();
    }

    let today_slots: Vec<&Value> = cuaca_groups
        .first()
        .and_then(|g| g.as_array())
        .map(|s| s.iter().collect())
        .unwrap_or_default();

    let first_slot = all_slots[0].1;
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
    let hu = first_slot["hu"].as_i64().unwrap_or(0);

    let first_code = first_slot["weather"].as_u64().unwrap_or(0) as u32;
    let icon = get_ascii_icon(first_code);

    let first_dt = first_slot["local_datetime"].as_str().unwrap_or("");
    let first_date = NaiveDateTime::parse_from_str(first_dt, "%Y-%m-%d %H:%M:%S")
        .map(|dt| {
            dt.date()
                .format_localized("%a %d %b", Locale::en_US)
                .to_string()
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    let mut out = String::new();

    out.push_str(&color::header(&format!("Weather Report: {}", location)));
    out.push('\n');
    out.push_str(&format!("{}\n", first_date));
    out.push('\n');
    out.push('\n');

    let unit = if fahrenheit { "°F" } else { "°C" };

    out.push_str(&format!(
        "     {}   {}\n",
        color::weather_icon_line(icon[0], first_code),
        color::desc_text(desc, first_code),
    ));
    out.push_str(&format!(
        "     {}  {}{}{}\n",
        color::weather_icon_line(icon[1], first_code),
        temp,
        if temp >= 0 { "+" } else { "" },
        unit,
    ));
    out.push_str(&format!(
        "     {}  {} {} {} km/h   {}   {:.1} mm   {}%\n",
        color::weather_icon_line(icon[2], first_code),
        format_wind_dir_icon(wd_deg),
        wd_cardinal,
        ws as i64,
        vs_text,
        tp,
        hu,
    ));
    out.push_str(&format!(
        "     {}\n",
        color::weather_icon_line(icon[3], first_code)
    ));
    out.push_str(&format!(
        "     {}\n",
        color::weather_icon_line(icon[4], first_code)
    ));
    out.push('\n');

    let temps: Vec<f64> = today_slots
        .iter()
        .map(|s| {
            let t = s["t"].as_f64().unwrap_or(0.0);
            if fahrenheit {
                celsius_to_fahrenheit(t as i64) as f64
            } else {
                t
            }
        })
        .collect();
    let rains: Vec<f64> = today_slots
        .iter()
        .map(|s| s["tp"].as_f64().unwrap_or(0.0))
        .collect();
    let humids: Vec<f64> = today_slots
        .iter()
        .map(|s| s["hu"].as_f64().unwrap_or(0.0))
        .collect();
    let winds: Vec<f64> = today_slots
        .iter()
        .map(|s| s["ws"].as_f64().unwrap_or(0.0))
        .collect();
    let clouds: Vec<f64> = today_slots
        .iter()
        .map(|s| s["tcc"].as_f64().unwrap_or(0.0))
        .collect();

    let vis_vals: Vec<f64> = today_slots
        .iter()
        .map(|s| {
            s["vs_text"]
                .as_str()
                .and_then(|v| {
                    v.split_whitespace()
                        .find_map(|token| token.parse::<f64>().ok())
                })
                .unwrap_or(0.0)
        })
        .collect();

    let times: Vec<String> = today_slots
        .iter()
        .map(|s| {
            let dt_str = s["local_datetime"].as_str().unwrap_or("");
            if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
                format!("{:02}", dt.hour())
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S") {
                format!("{:02}", dt.hour())
            } else if dt_str.len() >= 13 {
                let h = &dt_str[11..13];
                if h.parse::<u32>().is_ok() {
                    h.to_string()
                } else {
                    "??".to_string()
                }
            } else {
                "??".to_string()
            }
        })
        .collect();

    let height = args.yticks.clamp(3, 6);

    let mut temp_rows = temperature_panel(&temps, &times, height);
    let mut rain_rows = sparkline_panel(&rains, &times, height);
    let mut humid_rows = sparkline_panel(&humids, &times, height);
    let mut wind_rows = sparkline_panel(&winds, &times, height);
    let mut cloud_rows = sparkline_panel(&clouds, &times, height);
    let mut vis_rows = sparkline_panel(&vis_vals, &times, height);

    colorize_temp_panel(&mut temp_rows, &temps);
    colorize_spark_panel(&mut rain_rows, color::rain_bar);
    colorize_spark_panel(&mut humid_rows, color::humid_spark);
    colorize_spark_panel(&mut wind_rows, color::wind_spark);
    colorize_spark_panel(&mut cloud_rows, color::cloud_bar);
    colorize_spark_panel(&mut vis_rows, color::vis_spark);

    render_row(
        &mut out,
        &lang.temperature(),
        &temp_rows,
        &lang.rainfall(),
        &rain_rows,
        &lang.humidity_label(),
        &humid_rows,
    );
    out.push('\n');
    render_row(
        &mut out,
        &lang.wind_label(),
        &wind_rows,
        &lang.cloud_label(),
        &cloud_rows,
        &lang.visibility_label(),
        &vis_rows,
    );
    out.push('\n');

    let day_labels = [lang.today(), lang.tomorrow(), lang.day_after_tomorrow()];
    for (g_idx, group) in cuaca_groups.iter().enumerate() {
        let slots: Vec<&Value> = group.as_array().map_or(vec![], |s| s.iter().collect());
        if slots.is_empty() {
            continue;
        }

        let date_str = slots
            .first()
            .and_then(|s| s["local_datetime"].as_str())
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S").ok())
            .map(|dt| {
                dt.date()
                    .format_localized("%a %d %b", Locale::en_US)
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let label = day_labels.get(g_idx).unwrap_or(&"");
        let header = if !label.is_empty() {
            format!("{}, {}", label, date_str)
        } else {
            date_str.clone()
        };

        let day_temps: Vec<i64> = slots.iter().map(|s| s["t"].as_i64().unwrap_or(0)).collect();
        let min_t = day_temps.iter().min().copied().unwrap_or(0);
        let max_t = day_temps.iter().max().copied().unwrap_or(0);
        let total_rain: f64 = slots.iter().map(|s| s["tp"].as_f64().unwrap_or(0.0)).sum();
        let avg_humid: f64 = {
            let sum: i64 = slots.iter().map(|s| s["hu"].as_i64().unwrap_or(0)).sum();
            sum as f64 / slots.len() as f64
        };

        let desc_key = lang.weather_desc_key();
        let first_desc = slots[0][desc_key].as_str().unwrap_or("?");

        let (min_t, max_t) = if fahrenheit {
            (celsius_to_fahrenheit(min_t), celsius_to_fahrenheit(max_t))
        } else {
            (min_t, max_t)
        };

        let days_code = slots[0]["weather"].as_u64().unwrap_or(0) as u32;

        out.push_str(&format!(
            "  {}  {}  {}-{}°C  {:.1} mm {}  {:.0}% avg\n",
            header,
            color::desc_text(first_desc, days_code),
            min_t,
            max_t,
            total_rain,
            lang.total(),
            avg_humid,
        ));
    }

    out.push('\n');
    out.push_str(&color::dim(lang.source()));
    out.push('\n');

    out
}

fn render_row(
    out: &mut String,
    title1: &str,
    panel1: &[String],
    title2: &str,
    panel2: &[String],
    title3: &str,
    panel3: &[String],
) {
    let title_field = |t: &str| -> String {
        let t = if t.len() > 24 { &t[..24] } else { t };
        let pad_left = 6;
        let graph_w = 24;
        let total_w = pad_left + graph_w;
        let title_len = t.len();
        let pad = if title_len >= graph_w {
            0
        } else {
            (graph_w - title_len) / 2
        };
        let mut field = " ".repeat(pad_left + pad);
        field.push_str(t);
        let remaining = total_w - field.len();
        if remaining > 0 {
            field.push_str(&" ".repeat(remaining));
        }
        field
    };

    out.push_str(&format!(
        "{}  {}  {}\n",
        title_field(title1),
        title_field(title2),
        title_field(title3)
    ));

    let max_h = panel1.len().max(panel2.len()).max(panel3.len());
    for r in 0..max_h {
        let l1 = panel1
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l2 = panel2
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l3 = panel3
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        out.push_str(&format!("{}  {}  {}\n", l1, l2, l3));
    }
}

fn colorize_temp_panel(rows: &mut Vec<String>, temps: &[f64]) {
    let min_t = temps.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_t = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for i in 0..rows.len() {
        let rev_i = rows.len() - 1 - i;
        let temp_at_row = if rows.len() <= 1 {
            25.0
        } else {
            min_t + (max_t - min_t) * (rev_i as f64 / (rows.len() - 1) as f64)
        };
        rows[i] = color::temp_line(&rows[i], temp_at_row as i64);
    }
}

fn colorize_spark_panel(rows: &mut Vec<String>, color_fn: fn(&str) -> String) {
    for i in 0..rows.len() {
        rows[i] = color_fn(&rows[i]);
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
