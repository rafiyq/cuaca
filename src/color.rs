use crate::cli::ColorMode;
use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_color_mode(mode: ColorMode) {
    let enabled = match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            if std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
                false
            } else {
                std::io::stdout().is_terminal()
            }
        }
    };
    COLOR_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_color() -> bool {
    COLOR_ENABLED.load(Ordering::Relaxed)
}

const RESET: &str = "\x1b[0m";

fn paint(text: &str, code: &str) -> String {
    if !is_color() {
        return text.to_string();
    }
    format!("{}{}{}", code, text, RESET)
}

pub fn temp_line(text: &str, temp: i64) -> String {
    if !is_color() {
        return text.to_string();
    }
    let code = if temp <= 10 {
        "\x1b[38;5;33m"
    } else if temp <= 20 {
        "\x1b[38;5;51m"
    } else if temp <= 25 {
        "\x1b[38;5;49m"
    } else if temp <= 28 {
        "\x1b[38;5;226m"
    } else if temp <= 32 {
        "\x1b[38;5;214m"
    } else {
        "\x1b[38;5;196m"
    };
    format!("{}{}{}", code, text, RESET)
}

pub fn rain_bar(text: &str) -> String {
    paint(text, "\x1b[38;5;27m")
}

pub fn cloud_bar(text: &str) -> String {
    paint(text, "\x1b[38;5;244m")
}

pub fn humid_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;129m")
}

pub fn wind_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;46m")
}

pub fn vis_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;51m")
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm' (inclusive)
            while let Some(c2) = chars.next() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn weather_icon_line(line: &str, code: u32) -> String {
    if !is_color() {
        return strip_ansi(line);
    }
    // If line already contains ANSI codes, assume it's pre-colored and return as-is.
    if line.contains('\x1b') {
        return line.to_string();
    }
    let color_code = match code {
        0 | 1 => "\x1b[38;5;226m",
        2 => "\x1b[38;5;220m",
        3 | 4 => "\x1b[38;5;248m",
        5 | 10 | 45 => "\x1b[38;5;248m",
        60 | 61 => "\x1b[38;5;51m",
        63 | 80 => "\x1b[38;5;27m",
        95 | 97 => "\x1b[38;5;201m",
        _ => "\x1b[38;5;250m",
    };
    format!("{}{}{}", color_code, line, RESET)
}

pub fn desc_text(text: &str, code: u32) -> String {
    weather_icon_line(text, code)
}

pub fn header(text: &str) -> String {
    paint(text, "\x1b[1;37m")
}

pub fn dim(text: &str) -> String {
    paint(text, "\x1b[2m")
}

pub fn warning(text: &str) -> String {
    if !is_color() {
        return text.to_string();
    }
    paint(text, "\x1b[38;5;226m") // bright yellow
}
