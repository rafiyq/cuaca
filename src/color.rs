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
            for c2 in chars.by_ref() {
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

/// Compute hex color for ANSI 256-color index.
fn ansi256_to_hex(n: u8) -> String {
    // 0-15: standard 16 colors
    let basic: [&str; 16] = [
        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
        "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
    ];
    if n < 16 {
        return basic[n as usize].to_string();
    }
    // 16-231: 6x6x6 color cube
    if n < 232 {
        let idx = n as usize - 16;
        let r = idx / 36;
        let g = (idx % 36) / 6;
        let b = idx % 6;
        // Intensity values for each channel level 0..5
        let vals = [0, 95, 135, 175, 215, 255];
        let rv = vals[r];
        let gv = vals[g];
        let bv = vals[b];
        return format!("#{:02x}{:02x}{:02x}", rv, gv, bv);
    }
    // 232-255: grayscale ramp from #080808 to #eeeeee (24 shades)
    let shade = (n as usize - 232) * 10 + 8;
    format!("#{:02x}{:02x}{:02x}", shade, shade, shade)
}

/// Convert ANSI escape sequences (basic 256-color) to Pango markup.
/// Supports: reset (0), foreground 256-color (38;5;N). Ignores unsupported codes.
pub fn ansi_to_pango(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut current_fg: Option<u8> = None;

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // accumulate CSI sequence
            let mut seq = String::new();
            for c2 in chars.by_ref() {
                seq.push(c2);
                if c2 == 'm' {
                    break;
                }
            }
            if let Some(inner) = seq.strip_prefix('[') {
                // Strip trailing 'm' from CSI sequence
                let inner = inner.trim_end_matches('m');
                if inner.is_empty() || inner == "0" {
                    if current_fg.is_some() {
                        result.push_str("</span>");
                        current_fg = None;
                    }
                    continue;
                }
                let parts: Vec<Option<u8>> =
                    inner.split(';').map(|s| s.parse::<u8>().ok()).collect();
                let mut i = 0;
                while i < parts.len() {
                    match parts[i] {
                        Some(0) => {
                            if current_fg.is_some() {
                                result.push_str("</span>");
                                current_fg = None;
                            }
                        }
                        Some(38) => {
                            // check for 38;5;N
                            if i + 2 < parts.len() && parts[i + 1] == Some(5) {
                                if let Some(n) = parts[i + 2] {
                                    if current_fg != Some(n) {
                                        if current_fg.is_some() {
                                            result.push_str("</span>");
                                        }
                                        let hex = ansi256_to_hex(n);
                                        result.push_str(&format!(r#"<span foreground="{}">"#, hex));
                                        current_fg = Some(n);
                                    }
                                    i += 2;
                                }
                            }
                            // ignore 38;2 (truecolor)
                        }
                        Some(48) => {
                            // Background color, ignore
                            if i + 2 < parts.len() {
                                i += 2;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
        } else {
            result.push(c);
        }
    }
    if current_fg.is_some() {
        result.push_str("</span>");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_pango_256_color() {
        let hex = ansi256_to_hex(226);
        let input = "\x1b[38;5;226mHello\x1b[0m";
        let output = ansi_to_pango(input);
        assert!(output.contains(&format!("foreground=\"{}\"", hex)));
        assert!(output.contains("Hello"));
        assert!(!output.contains('\x1b'));
    }

    #[test]
    fn test_ansi_to_pango_reset() {
        let input = "\x1b[0mNormal";
        let output = ansi_to_pango(input);
        assert_eq!(output, "Normal");
    }

    #[test]
    fn test_ansi_to_pango_multiple_colors() {
        let hex1 = ansi256_to_hex(226);
        let hex2 = ansi256_to_hex(51);
        let input = "\x1b[38;5;226mRedish\x1b[38;5;51mBlueish\x1b[0mEnd";
        let output = ansi_to_pango(input);
        assert!(output.contains(&format!("foreground=\"{}\"", hex1)));
        assert!(output.contains(&format!("foreground=\"{}\"", hex2)));
        assert!(output.contains("Redish"));
        assert!(output.contains("Blueish"));
        assert!(output.contains("End"));
    }

    #[test]
    fn test_ansi_to_pango_ignore_unsupported() {
        // Truecolor (38;2) should be ignored; we strip the escape and output plain text.
        let input = "\x1b[38;2;255;0;0mTrueColor\x1b[0m";
        let output = ansi_to_pango(input);
        assert_eq!(output, "TrueColor");
    }
}
