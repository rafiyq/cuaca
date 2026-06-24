use super::*;
use crate::cli::args::{Args, OutputFormat};
use crate::color::{set_color_mode, ColorMode};
use serde_json::json;

fn default_args() -> Args {
    Args {
        adm4: None,
        lat: None,
        lon: None,
        name: None,
        lang: crate::lang::Lang::EN,
        format: OutputFormat::Text,
        custom_indicator: None,
        date_format: "%a %d %b".to_string(),
        ampm: false,
        nerd: false,
        hide_details: false,
        fahrenheit: false,
        warnings: false,
        warnings_ttl: 15,
        yticks: 4,
        color: ColorMode::Never,
        raw: false,
    }
}

#[test]
fn test_render_terminal_no_data() {
    set_color_mode(ColorMode::Never);
    let args = default_args();
    let weather = json!({ "lokasi": {}, "data": [] });
    let warnings = vec![];
    let output = render_terminal(&weather, &args, &warnings);
    assert_eq!(output, "No forecast data available");
}

#[test]
fn test_render_row() {
    let title1 = "Temp";
    let title2 = "Rain";
    let title3 = "Wind";
    let panel1 = vec![" 25°C".to_string()];
    let panel2 = vec!["0.0 mm".to_string()];
    let panel3 = vec!["5 km/h".to_string()];
    let mut out = String::new();
    render_row(&mut out, title1, &panel1, title2, &panel2, title3, &panel3);
    assert!(out.contains(title1));
    assert!(out.contains(title2));
    assert!(out.contains(title3));
}

#[test]
fn test_colorize_temp_panel() {
    let temps = vec![20.0, 30.0];
    let mut rows = vec!["25°C".to_string(), "26°C".to_string()];
    colorize_temp_panel(&mut rows, &temps);
    assert_eq!(rows.len(), 2);
}
