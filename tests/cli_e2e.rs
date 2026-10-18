//! End-to-end CLI tests: resolve → cached forecast → Waybar render.

use clap::Parser;
use cuaca_cli::cli::args::Args;
use cuaca_cli::cli::run::run_internal;
use serde_json::json;

fn bmkg_forecast() -> serde_json::Value {
    json!({
        "lokasi": { "adm4": "31.71.03.1001", "desa": "Kemayoran",
            "provinsi": "DKI Jakarta", "kotkab": "Jakarta Pusat",
            "kecamatan": "Kemayoran", "lat": -6.17, "lon": 106.84 },
        "data": [ { "cuaca": [
            [
                { "local_datetime": "2026-05-10 13:00:00", "utc_datetime": "2026-05-10 06:00:00",
                  "weather": 1, "weather_desc": "Cerah", "weather_desc_en": "Sunny",
                  "t": 31, "hu": 60, "tcc": 20, "tp": 0.0, "ws": 8.0,
                  "wd": "NE", "wd_deg": 45, "vs_text": "< 9 km" },
                { "local_datetime": "2026-05-10 16:00:00", "utc_datetime": "2026-05-10 09:00:00",
                  "weather": 2, "weather_desc": "Cerah Berawan", "weather_desc_en": "Partly cloudy",
                  "t": 30, "hu": 65, "tcc": 40, "tp": 0.2, "ws": 10.0,
                  "wd": "E", "wd_deg": 90, "vs_text": "9 - 12 km" }
            ]
        ] } ]
    })
}

#[test]
#[serial_test::serial]
fn test_resolve_to_waybar_json_from_cache() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("CUACA_CACHE_DIR", dir.path());

    let adm4 = "31.71.03.1001";
    let wrapper = json!({
        "fetched_at": chrono::Utc::now().to_rfc3339(),
        "adm4": adm4,
        "forecast": bmkg_forecast()
    });
    let cachefile = dir.path().join(format!("cuaca-{}.json", adm4));
    std::fs::write(&cachefile, serde_json::to_string_pretty(&wrapper).unwrap()).unwrap();

    let args = Args::parse_from(["cuaca", "--adm4", adm4, "--lang", "id"]);
    let out = run_internal(&args, None).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(parsed["text"].as_str().unwrap().contains("31\u{00b0}"));
    assert_eq!(parsed["class"], "cerah");
    let tooltip = parsed["tooltip"].as_str().unwrap();
    assert!(tooltip.contains("<b>Prakiraan Cuaca:</b> Kemayoran"));
    assert!(tooltip.contains("Cerah Berawan"));
}

#[test]
#[serial_test::serial]
fn test_forecast_override_text_format() {
    let args = Args::parse_from([
        "cuaca",
        "--format",
        "text",
        "--lang",
        "en",
        "--color",
        "never",
        "--adm4",
        "31.71.03.1001",
    ]);
    let out = run_internal(&args, Some(bmkg_forecast())).unwrap();
    assert!(out.contains("Weather Report"));
    assert!(out.contains("Sunny"));
    assert!(out.contains("+31(31) \u{00b0}C"));
}

#[test]
#[serial_test::serial]
fn test_forecast_override_bar_with_defaults() {
    let args = Args::parse_from(["cuaca", "--lang", "id"]);
    let out = run_internal(&args, Some(bmkg_forecast())).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(parsed["text"].as_str().unwrap().contains("31\u{00b0}"));
}
