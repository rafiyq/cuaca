//! Orchestration of the application logic.

use super::args::{Args, OutputFormat};
use crate::error::CuacaError;
use crate::location;
use crate::warnings;
use crate::weather::ensure_forecast;
use chrono::Utc;
use serde_json::Value;

/// Internal runner that returns the output as a string (for daemon and client).
/// If `forecast_override` is provided, it bypasses location resolution and weather fetching.
pub fn run_internal(args: &Args, forecast_override: Option<Value>) -> Result<String, CuacaError> {
    // set color mode
    cuaca::color::set_color_mode(args.color);

    // Get forecast: either from override or by fetching
    let weather = if let Some(fc) = forecast_override {
        fc
    } else {
        // Resolve location
        let adm4 = location::resolve(
            args.adm4.as_deref(),
            args.lat,
            args.lon,
            args.name.as_deref(),
        )?;
        // Ensure forecast via cache/network (10 min TTL, no archive for direct/client)
        ensure_forecast(&adm4, 600, false)?
    };

    // If raw flag is set, output the raw forecast JSON and exit
    if args.raw {
        return Ok(serde_json::to_string_pretty(&weather)?);
    }

    // Parse the raw BMKG payload into the library's typed model
    let forecast = cuaca::Forecast::from_value(weather.clone())?;

    // Fetch warnings if requested; relevance (polygon containment with
    // province name + validity-time fallback) is decided here via the library.
    let warnings_list: Vec<cuaca::Warning> = if args.warnings {
        let province = weather["lokasi"]["provinsi"].as_str().unwrap_or("");
        let lat = weather["lokasi"]["lat"].as_f64().unwrap_or(0.0);
        let lon = weather["lokasi"]["lon"].as_f64().unwrap_or(0.0);
        let alerts = warnings::fetch_warnings(args.lang, args.warnings_ttl).unwrap_or_default();
        let now = Utc::now();
        cuaca::warnings::select_relevant(&alerts, lat, lon, province, now)
            .into_iter()
            .cloned()
            .collect()
    } else {
        vec![]
    };

    let cfg = cuaca::Config::from(args);

    let output = match args.format {
        OutputFormat::Bar => cuaca::waybar::render(&cfg, &forecast, &warnings_list)?.to_json()?,
        OutputFormat::Text => cuaca::terminal::render(&cfg, &forecast, &warnings_list),
        OutputFormat::Json => serde_json::to_string_pretty(&weather)?,
    };

    Ok(output)
}

/// Public entry point that prints result.
pub fn run(args: Args) -> Result<(), CuacaError> {
    let output = run_internal(&args, None)?;
    println!("{}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_from_args_to_config() {
        let args = Args::parse_from([
            "cuaca",
            "--lang",
            "id",
            "--format",
            "text",
            "--custom-indicator",
            "{T}",
            "--date-format",
            "%d-%m-%Y",
            "--ampm",
            "--nerd",
            "--hide-details",
            "--fahrenheit",
            "--yticks",
            "5",
            "--color",
            "never",
            "--adm4",
            "31.71.03.1001",
        ]);
        let cfg = cuaca::Config::from(&args);
        assert_eq!(cfg.lang, cuaca::Lang::ID);
        assert_eq!(cfg.format, cuaca::OutputFormat::Text);
        assert_eq!(cfg.custom_indicator.as_deref(), Some("{T}"));
        assert_eq!(cfg.date_format, "%d-%m-%Y");
        assert!(cfg.ampm && cfg.nerd && cfg.hide_details && cfg.fahrenheit);
        assert_eq!(cfg.yticks, 5);
        assert_eq!(cfg.color, cuaca::ColorMode::Never);
    }

    #[test]
    fn test_args_serde_roundtrip() {
        let args = Args::parse_from([
            "cuaca",
            "--adm4",
            "31.71.03.1001",
            "--nerd",
            "--format",
            "text",
        ]);
        let json = serde_json::to_string(&args).unwrap();
        let back: Args = serde_json::from_str(&json).unwrap();
        assert_eq!(back.adm4.as_deref(), Some("31.71.03.1001"));
        assert!(back.nerd);
        assert_eq!(back.format, cuaca::OutputFormat::Text);
    }

    #[test]
    fn test_run_internal_raw_passthrough() {
        let args = Args::parse_from([
            "cuaca",
            "--raw",
            "--format",
            "json",
            "--lang",
            "en",
            "--adm4",
            "31.71.03.1001",
        ]);
        let weather = serde_json::json!({ "lokasi": { "provinsi": "DKI Jakarta" }, "data": [] });
        let out = run_internal(&args, Some(weather.clone())).unwrap();
        assert_eq!(out, serde_json::to_string_pretty(&weather).unwrap());
    }

    #[test]
    fn test_run_internal_json_format_pretty_prints_raw() {
        let args = Args::parse_from(["cuaca", "--format", "json"]);
        let weather = serde_json::json!({ "lokasi": { "provinsi": "DKI Jakarta" } });
        let out = run_internal(&args, Some(weather.clone())).unwrap();
        assert_eq!(out, serde_json::to_string_pretty(&weather).unwrap());
    }

    #[test]
    fn test_run_internal_invalid_forecast_errors() {
        let args = Args::parse_from(["cuaca", "--format", "bar", "--lang", "en"]);
        let weather = serde_json::json!({ "unexpected": true });
        assert!(run_internal(&args, Some(weather)).is_err());
    }
}
