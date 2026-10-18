//! Forecast archive statistics: reads the JSONL archive, feeds
//! `cuaca::stats::summarize`, and prints a table or JSON.

use crate::cache::cache_dir;
use crate::cli::args::{OutputFormat, StatsOpts};
use chrono::{DateTime, Utc};
use cuaca::Forecast;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Compute rows from the archives and render the requested output as a string.
pub fn compute_output(opts: StatsOpts) -> Result<String, Box<dyn std::error::Error>> {
    let archive_path = cache_dir().join("forecasts.jsonl");
    let file = File::open(&archive_path)
        .map_err(|e| format!("could not open archive {}: {}", archive_path.display(), e))?;
    let reader = BufReader::new(file);

    let mut entries: Vec<cuaca::stats::ArchiveEntry> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        // Parse wrapper
        let wrapper: serde_json::Value = serde_json::from_str(&line)?;
        let fetched_at_str = wrapper["fetched_at"].as_str().ok_or("missing fetched_at")?;
        let fetched_at = DateTime::parse_from_rfc3339(fetched_at_str)?.with_timezone(&Utc);
        let adm4 = wrapper["adm4"].as_str().ok_or("missing adm4")?.to_string();
        let forecast = Forecast::from_value(wrapper["forecast"].clone())?;
        entries.push(cuaca::stats::ArchiveEntry {
            fetched_at,
            adm4,
            forecast,
        });
    }

    let variables: Vec<String> = opts
        .variables
        .split(',')
        .map(|v| v.trim().to_string())
        .collect();
    let lib_opts = cuaca::stats::StatsOpts {
        adm4: opts.adm4,
        start: opts.start,
        end: opts.end,
        variables: variables.clone(),
    };

    let rows = cuaca::stats::summarize(&entries, &lib_opts);
    Ok(match opts.format {
        Some(OutputFormat::Json) => cuaca::stats::to_json(&rows)?,
        _ => cuaca::stats::to_table(&rows, &variables),
    })
}

/// Compute statistics from the forecasts archive and print to stdout.
pub fn compute_and_print(opts: StatsOpts) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", compute_output(opts)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn slot(hour: u32, t: i64, hu: i64) -> serde_json::Value {
        let dt = format!("2026-06-01 {:02}:00:00", hour);
        json!({ "local_datetime": dt, "utc_datetime": dt, "t": t, "hu": hu,
                "tcc": 50, "tp": 0.5, "ws": 10.0, "wd": "NE", "wd_deg": 45,
                "weather": 1, "weather_desc": "Cerah", "weather_desc_en": "Sunny" })
    }

    fn wrapper(fetched_at: &str, adm4: &str, t: i64, hu: i64) -> serde_json::Value {
        json!({ "fetched_at": fetched_at, "adm4": adm4,
                "forecast": { "lokasi": { "adm4": adm4 }, "data": [ { "cuaca": [ [ slot(12, t, hu) ] ] } ] } })
    }

    fn opts(format: Option<crate::cli::args::OutputFormat>) -> StatsOpts {
        StatsOpts {
            adm4: Some("31.71.03.1001".to_string()),
            start: None,
            end: None,
            variables: "t,hu,tp,ws,tcc".to_string(),
            format,
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_stats_table_output() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CUACA_CACHE_DIR", dir.path());
        let rows = vec![
            wrapper("2026-05-30T00:00:00Z", "31.71.03.1001", 30, 70),
            wrapper("2026-05-31T00:00:00Z", "31.71.03.1001", 32, 80),
        ];
        let mut text = String::new();
        for r in &rows {
            text += &serde_json::to_string(r).unwrap();
            text.push('\n');
        }
        std::fs::write(dir.path().join("forecasts.jsonl"), text).unwrap();
        let out = compute_output(opts(None)).unwrap();
        assert!(out.contains("t 31.0±1.4"), "got: {}", out);
        assert!(out.contains("2026-06-01"), "got: {}", out);
    }

    #[test]
    #[serial_test::serial]
    fn test_stats_json_output() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CUACA_CACHE_DIR", dir.path());
        let mut text =
            serde_json::to_string(&wrapper("2026-05-30T00:00:00Z", "31.71.03.1001", 30, 70))
                .unwrap();
        text.push('\n');
        std::fs::write(dir.path().join("forecasts.jsonl"), text).unwrap();

        let out = compute_output(opts(Some(crate::cli::args::OutputFormat::Json))).unwrap();
        assert!(out.contains("lead_avg"), "got: {}", out);
        assert!(out.contains("hu"), "got: {}", out);
    }
}
