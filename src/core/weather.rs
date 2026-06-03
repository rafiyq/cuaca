use crate::core::error::CuacaError;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Save weather JSON to cache file. Errors are returned and can be ignored if desired.
pub fn save_cache(cachefile: &PathBuf, weather: &Value) -> Result<(), CuacaError> {
    if let Some(parent) = cachefile.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(cachefile)?;
    let content = serde_json::to_string_pretty(weather)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Fetch weather with exponential backoff. Returns parsed JSON or error.
pub fn fetch_weather(
    client: &Client,
    url: &str,
    cachefile: &PathBuf,
    iterations: &mut usize,
    threshold: usize,
) -> Result<Value, CuacaError> {
    loop {
        match client.get(url).send() {
            Ok(response) => match response.json::<Value>() {
                Ok(json) => {
                    let _ = save_cache(cachefile, &json);
                    return Ok(json);
                }
                Err(e) => return Err(CuacaError::Data(format!("JSON parse error: {}", e))),
            },
            Err(e) => {
                *iterations += 1;
                if *iterations >= threshold {
                    return Err(CuacaError::Network(e));
                }
                thread::sleep(Duration::from_millis(500 * *iterations as u64));
            }
        }
    }
}
