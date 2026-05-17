use chrono::{NaiveDateTime, Timelike};

use crate::constants::{
    FALLBACK_ICON, FALLBACK_ICON_NERD, ICON_PLACEHOLDER, WEATHER_CODES, WEATHER_CODES_NERD,
};

pub fn format_time(local_datetime: &str, ampm: bool) -> String {
    let dt =
        NaiveDateTime::parse_from_str(local_datetime, "%Y-%m-%d %H:%M:%S").unwrap_or_else(|_| {
            NaiveDateTime::parse_from_str("2000-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        });
    let hour = dt.time().hour();
    if ampm {
        let am_or_pm = if hour >= 12 { "pm" } else { "am" };
        let hour12 = if hour == 0 || hour == 12 {
            12
        } else {
            hour % 12
        };
        format!("{: <4}", format!("{}{}", hour12, am_or_pm))
    } else {
        format!("{:02}:{:02}", hour, dt.time().minute())
    }
}

pub fn format_temp(temp: i64) -> String {
    format!("{: >3}\u{00b0}", temp)
}

pub fn celsius_to_fahrenheit(celsius: i64) -> i64 {
    (celsius as f64 * 9.0 / 5.0 + 32.0).round() as i64
}

pub fn format_wind_dir_icon(degrees: i64, nerd: bool) -> &'static str {
    let dir = ((degrees % 360) as f64 / 45.0).round() as usize % 8;
    if nerd {
        [
            "\u{F0340}",
            "\u{F0347}",
            "\u{F0343}",
            "\u{F0349}",
            "\u{F0341}",
            "\u{F0348}",
            "\u{F0342}",
            "\u{F0346}",
        ][dir]
    } else {
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
}

pub fn get_weather_icon(code: u32, nerd: bool) -> &'static str {
    let table = if nerd {
        WEATHER_CODES_NERD
    } else {
        WEATHER_CODES
    };
    table
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, s)| *s)
        .unwrap_or(if nerd {
            FALLBACK_ICON_NERD
        } else {
            FALLBACK_ICON
        })
}

pub fn format_indicator(slot: &serde_json::Value, expression: &str, weather_icon: &str) -> String {
    let map = match slot.as_object() {
        Some(m) => m,
        None => return String::new(),
    };

    let mut result = expression.to_string();

    for (key, value) in map {
        let placeholder = format!("{{{}}}", key);
        if result.contains(&placeholder) {
            let formatted = if value.is_number() {
                value.to_string()
            } else {
                value.as_str().unwrap_or("").to_string()
            };

            result = result.replace(&placeholder, &formatted);
        }
    }

    if result.contains(ICON_PLACEHOLDER) {
        result = result.replace(ICON_PLACEHOLDER, weather_icon);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_time_24h() {
        assert_eq!(format_time("2026-05-10 13:00:00", false), "13:00");
        assert_eq!(format_time("2026-05-10 00:00:00", false), "00:00");
        assert_eq!(format_time("2026-05-10 06:30:00", false), "06:30");
    }

    #[test]
    fn format_time_ampm() {
        assert_eq!(format_time("2026-05-10 13:00:00", true).trim(), "1pm");
        assert_eq!(format_time("2026-05-10 00:00:00", true).trim(), "12am");
        assert_eq!(format_time("2026-05-10 08:00:00", true).trim(), "8am");
        assert_eq!(format_time("2026-05-10 12:00:00", true).trim(), "12pm");
    }

    #[test]
    fn format_temp_pads_to_three_chars() {
        assert_eq!(format_temp(5), "  5\u{00b0}");
        assert_eq!(format_temp(20), " 20\u{00b0}");
        assert_eq!(format_temp(100), "100\u{00b0}");
        assert_eq!(format_temp(-3), " -3\u{00b0}");
    }

    #[test]
    fn format_wind_dir_all_cardinals() {
        assert_eq!(format_wind_dir_icon(0, false), "\u{2b06}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(45, false), "\u{2197}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(90, false), "\u{27a1}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(135, false), "\u{2198}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(180, false), "\u{2b07}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(225, false), "\u{2199}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(270, false), "\u{2b05}\u{fe0f}");
        assert_eq!(format_wind_dir_icon(315, false), "\u{2196}\u{fe0f}");
    }

    #[test]
    fn format_wind_dir_all_nerd() {
        assert_eq!(format_wind_dir_icon(0, true), "\u{F0340}");
        assert_eq!(format_wind_dir_icon(90, true), "\u{F0343}");
        assert_eq!(format_wind_dir_icon(180, true), "\u{F0341}");
        assert_eq!(format_wind_dir_icon(270, true), "\u{F0342}");
    }

    #[test]
    fn get_weather_icon_known_codes() {
        assert_eq!(get_weather_icon(0, false), "\u{2600}\u{fe0f}");
        assert_eq!(get_weather_icon(2, false), "\u{26c5}");
        assert_eq!(get_weather_icon(3, false), "\u{2601}\u{fe0f}");
    }

    #[test]
    fn get_weather_icon_fallback() {
        assert_eq!(get_weather_icon(99, false), FALLBACK_ICON);
        assert_eq!(get_weather_icon(99, true), FALLBACK_ICON_NERD);
    }

    fn make_slot() -> serde_json::Value {
        json!({
            "t": 31,
            "hu": 68,
            "tcc": 39,
            "tp": 0.2,
            "ws": 8.7,
            "wd_deg": 14,
            "wd": "N",
            "wd_to": "S",
            "vs": 8401,
            "vs_text": "< 9 km",
            "weather": 1,
            "weather_desc": "Cerah",
            "weather_desc_en": "Sunny",
            "image": "https://api-apps.bmkg.go.id/storage/icon/cuaca/cerah-am.svg",
            "datetime": "2026-05-10T06:00:00Z",
            "local_datetime": "2026-05-10 13:00:00",
            "analysis_date": "2026-05-10T00:00:00",
            "time_index": "5-6"
        })
    }

    #[test]
    fn custom_indicator_substitutes_placeholder() {
        let slot = make_slot();
        let result = format_indicator(&slot, "{t}\u{00b0}C", "\u{2600}\u{fe0f}");
        assert_eq!(result, "31\u{00b0}C");
    }

    #[test]
    fn custom_indicator_substitutes_icon() {
        let slot = make_slot();
        let result = format_indicator(&slot, "{ICON} {t}\u{00b0}C", "\u{2600}\u{fe0f}");
        assert_eq!(result, "\u{2600}\u{fe0f} 31\u{00b0}C");
    }

    #[test]
    fn custom_indicator_multiple_placeholders() {
        let slot = make_slot();
        let result = format_indicator(&slot, "{ICON}{t}\u{00b0}C {weather_desc_en}", "\u{26c5}");
        assert_eq!(result, "\u{26c5}31\u{00b0}C Sunny");
    }

    #[test]
    fn custom_indicator_null_returns_empty() {
        let slot = json!(null);
        let result = format_indicator(&slot, "{t}", "\u{2600}\u{fe0f}");
        assert_eq!(result, "");
    }

    #[test]
    fn custom_indicator_number_placeholder() {
        let slot = json!({"t": 25, "hu": 70});
        let result = format_indicator(&slot, "{t} {hu}%", "\u{2600}\u{fe0f}");
        assert_eq!(result, "25 70%");
    }

    #[test]
    fn celsius_to_fahrenheit_conversions() {
        assert_eq!(celsius_to_fahrenheit(0), 32);
        assert_eq!(celsius_to_fahrenheit(100), 212);
        assert_eq!(celsius_to_fahrenheit(-40), -40);
        assert_eq!(celsius_to_fahrenheit(37), 99);
        assert_eq!(celsius_to_fahrenheit(20), 68);
    }
}
