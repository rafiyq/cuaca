use crate::core::cache::cache_dir;
use chrono::{NaiveDate, Timelike};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Agg {
    count: u64,
    sum: f64,
    sum_sq: f64,
    lead_sum: f64,
}

impl Agg {
    fn new() -> Self {
        Agg {
            count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            lead_sum: 0.0,
        }
    }

    fn update(&mut self, value: f64, lead_hours: f64) {
        self.count += 1;
        self.sum += value;
        self.sum_sq += value * value;
        self.lead_sum += lead_hours;
    }

    fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    fn stddev(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            let variance =
                (self.sum_sq - self.sum * self.sum / self.count as f64) / (self.count as f64 - 1.0);
            variance.sqrt()
        }
    }

    fn mean_lead(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.lead_sum / self.count as f64
        }
    }
}

/// Compute statistics from the forecasts archive and print to stdout.
pub fn compute_and_print(
    opts: super::cli::args::StatsOpts,
) -> Result<(), Box<dyn std::error::Error>> {
    let adm4_filter = opts.adm4;
    let start = opts.start;
    let end = opts.end;
    let variables_str = opts.variables.as_str();
    let format = opts.format;

    let cache_dir = cache_dir();
    let archive_path = cache_dir.join("forecasts.jsonl");
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);

    // Map: (date, hour) -> (var_name -> Agg)
    let mut data: HashMap<(NaiveDate, u16), HashMap<String, Agg>> = HashMap::new();

    // For lead calculation we can derive from any Agg later.

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        // Parse wrapper
        let wrapper: serde_json::Value = serde_json::from_str(&line)?;
        let fetched_at_str = wrapper["fetched_at"].as_str().ok_or("missing fetched_at")?;
        let fetched_at_utc = chrono::DateTime::parse_from_rfc3339(fetched_at_str)?; // DateTime<Utc>
        let adm4 = wrapper["adm4"].as_str().ok_or("missing adm4")?;
        if let Some(ref filter) = adm4_filter {
            if adm4 != filter {
                continue;
            }
        }
        let forecast = &wrapper["forecast"];

        // Iterate over slots
        if let Some(data_arr) = forecast["data"].as_array() {
            for day_group in data_arr {
                if let Some(cuaca_arr) = day_group["cuaca"].as_array() {
                    for slot in cuaca_arr {
                        // utc_datetime for lead
                        let utc_str = slot["utc_datetime"]
                            .as_str()
                            .ok_or("slot missing utc_datetime")?;
                        let slot_utc =
                            chrono::NaiveDateTime::parse_from_str(utc_str, "%Y-%m-%d %H:%M:%S")?;
                        let lead_secs = (fetched_at_utc.naive_utc() - slot_utc).num_seconds();
                        let lead_hours = lead_secs as f64 / 3600.0;

                        // local_datetime for grouping
                        let local_str = slot["local_datetime"]
                            .as_str()
                            .ok_or("slot missing local_datetime")?;
                        let local_dt =
                            chrono::NaiveDateTime::parse_from_str(local_str, "%Y-%m-%d %H:%M:%S")?;
                        let slot_date = local_dt.date();
                        let slot_hour = local_dt.time().hour() as u16;

                        // Date filters
                        if let Some(start_date) = start {
                            if slot_date < start_date {
                                continue;
                            }
                        }
                        if let Some(end_date) = end {
                            if slot_date > end_date {
                                continue;
                            }
                        }

                        // Extract numeric variables
                        let mut values = HashMap::new();
                        if let Some(t) = slot["t"].as_i64() {
                            values.insert("t", t as f64);
                        }
                        if let Some(hu) = slot["hu"].as_i64() {
                            values.insert("hu", hu as f64);
                        }
                        if let Some(tcc) = slot["tcc"].as_i64() {
                            values.insert("tcc", tcc as f64);
                        }
                        if let Some(tp) = slot["tp"].as_f64() {
                            values.insert("tp", tp);
                        }
                        if let Some(ws) = slot["ws"].as_f64() {
                            values.insert("ws", ws);
                        }

                        // Which variables to include?
                        let vars: Vec<&str> = variables_str.split(',').collect();
                        let key = (slot_date, slot_hour);
                        let entry = data.entry(key).or_default();
                        for var in &vars {
                            if let Some(&val) = values.get(var) {
                                let agg = entry.entry(var.to_string()).or_insert_with(Agg::new);
                                agg.update(val, lead_hours);
                            }
                        }
                    }
                }
            }
        }
    }

    // Prepare rows sorted by date then hour
    let mut rows: Vec<_> = data
        .iter()
        .map(|((date, hour), var_map)| {
            let mut row = StatRow {
                date: date.format("%Y-%m-%d").to_string(),
                hour: *hour as i32,
                lead_avg: 0.0,
                vars: HashMap::new(),
            };
            // Determine lead_avg from any variable's Agg (they share same lead stats)
            if let Some(first_agg) = var_map.values().next() {
                row.lead_avg = first_agg.mean_lead();
            }
            for (var, agg) in var_map {
                row.vars.insert(
                    var.clone(),
                    VarStats {
                        mean: agg.mean(),
                        stddev: agg.stddev(),
                        n: agg.count,
                    },
                );
            }
            row
        })
        .collect();
    rows.sort_by_key(|r| (r.date.clone(), r.hour));

    // Output
    match format.unwrap_or(crate::cli::args::OutputFormat::Text) {
        crate::cli::args::OutputFormat::Text => {
            // Header
            println!(
                "{:<12} {:<6} {:<8} {}",
                "Date",
                "Hour",
                "Lead(h)",
                vars_header(variables_str)
            );
            println!("{}", "-".repeat(12 + 1 + 6 + 1 + 8 + 1 + 50));
            for row in &rows {
                let mut var_parts = Vec::new();
                for var in variables_str.split(',') {
                    if let Some(stats) = row.vars.get(var) {
                        let part = format!("{} {:.1}±{:.1}", var, stats.mean, stats.stddev());
                        var_parts.push(part);
                    } else {
                        var_parts.push(format!("{} N/A", var));
                    }
                }
                let var_str = var_parts.join("  ");
                println!(
                    "{:<12} {:<6} {:<8.1} {}",
                    row.date, row.hour, row.lead_avg, var_str
                );
            }
        }
        crate::cli::args::OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct RowOut {
                date: String,
                hour: i32,
                lead_avg: f64,
                #[serde(flatten)]
                vars: HashMap<String, VarStatsOut>,
            }
            #[derive(serde::Serialize)]
            struct VarStatsOut {
                mean: f64,
                stddev: f64,
                n: u64,
            }
            let out: Vec<_> = rows
                .iter()
                .map(|row| RowOut {
                    date: row.date.clone(),
                    hour: row.hour,
                    lead_avg: row.lead_avg,
                    vars: row
                        .vars
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                VarStatsOut {
                                    mean: v.mean,
                                    stddev: v.stddev,
                                    n: v.n,
                                },
                            )
                        })
                        .collect(),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            // For text format, just use table
            println!(
                "{:<12} {:<6} {:<8} {}",
                "Date",
                "Hour",
                "Lead(h)",
                vars_header(variables_str)
            );
            println!("{}", "-".repeat(12 + 1 + 6 + 1 + 8 + 1 + 50));
            for row in &rows {
                let mut var_parts = Vec::new();
                for var in variables_str.split(',') {
                    if let Some(stats) = row.vars.get(var) {
                        let part = format!("{} {:.1}±{:.1}", var, stats.mean, stats.stddev());
                        var_parts.push(part);
                    } else {
                        var_parts.push(format!("{} N/A", var));
                    }
                }
                let var_str = var_parts.join("  ");
                println!(
                    "{:<12} {:<6} {:<8.1} {}",
                    row.date, row.hour, row.lead_avg, var_str
                );
            }
        }
    }

    Ok(())
}

fn vars_header(vars_str: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for v in vars_str.split(',') {
        parts.push(match v {
            "t" => "Temp(°C)".to_string(),
            "hu" => "Hum(%)".to_string(),
            "tp" => "Precip(mm)".to_string(),
            "ws" => "Wind(km/h)".to_string(),
            "tcc" => "Cloud(%)".to_string(),
            _ => v.to_string(),
        });
    }
    parts.join("  ")
}

struct StatRow {
    date: String,
    hour: i32,
    lead_avg: f64,
    vars: HashMap<String, VarStats>,
}

struct VarStats {
    mean: f64,
    stddev: f64,
    n: u64,
}

impl VarStats {
    fn stddev(&self) -> f64 {
        if self.n < 2 {
            0.0
        } else {
            self.stddev
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agg_mean() {
        let mut agg = Agg::new();
        agg.update(10.0, 1.0);
        agg.update(20.0, 2.0);
        agg.update(30.0, 3.0);
        assert_eq!(agg.mean(), 20.0);
        assert_eq!(agg.count, 3);
        assert_eq!(agg.mean_lead(), 2.0);
    }

    #[test]
    fn test_agg_stddev() {
        let mut agg = Agg::new();
        agg.update(2.0, 0.0);
        agg.update(4.0, 0.0);
        agg.update(6.0, 0.0);
        // variance = ((4-4)+(0)+(4))/2 = 4, stddev=2
        assert!((agg.stddev() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_agg_mean_lead() {
        let mut agg = Agg::new();
        agg.update(0.0, 1.0);
        agg.update(0.0, 3.0);
        agg.update(0.0, 5.0);
        assert_eq!(agg.mean_lead(), 3.0);
    }
}
