use crate::core::error::CuacaError;
use reqwest::blocking::Client;
use std::env;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BREAKER_THRESHOLD: u8 = 5;
const BREAKER_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RETRIES: u8 = 3;

struct CircuitBreaker {
    state: AtomicU8, // 0=Closed, 1=Open, 2=HalfOpen
    failures: AtomicU8,
    last_trip: AtomicU64,
}

impl CircuitBreaker {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(0), // Closed
            failures: AtomicU8::new(0),
            last_trip: AtomicU64::new(0),
        }
    }

    fn before_request(&self) -> Result<(), &'static str> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        loop {
            let s = self.state.load(Ordering::Relaxed);
            match s {
                0 => return Ok(()), // Closed
                1 => {
                    let last = self.last_trip.load(Ordering::Relaxed);
                    if now - last >= BREAKER_COOLDOWN.as_secs() {
                        if self
                            .state
                            .compare_exchange(1, 2, Ordering::Relaxed, Ordering::Relaxed)
                            .is_ok()
                        {
                            return Ok(());
                        }
                        continue;
                    } else {
                        return Err("API temporarily unavailable");
                    }
                }
                2 => return Ok(()), // HalfOpen
                _ => unreachable!(),
            }
        }
    }

    fn record_success(&self) {
        self.state.store(0, Ordering::Relaxed);
        self.failures.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        let mut failures = self.failures.load(Ordering::Relaxed);
        failures += 1;
        self.failures.store(failures, Ordering::Relaxed);

        if self.state.load(Ordering::Relaxed) == 2 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.last_trip.store(now, Ordering::Relaxed);
            self.state.store(1, Ordering::Relaxed);
            return;
        }

        if failures >= BREAKER_THRESHOLD {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.last_trip.store(now, Ordering::Relaxed);
            self.state.store(1, Ordering::Relaxed);
        }
    }
}

static BREAKER: CircuitBreaker = CircuitBreaker::new();

#[cfg(test)]
pub fn reset_breaker() {
    use std::sync::atomic::Ordering;
    BREAKER.state.store(0, Ordering::Relaxed);
    BREAKER.failures.store(0, Ordering::Relaxed);
    BREAKER.last_trip.store(0, Ordering::Relaxed);
}

fn get_base_url() -> String {
    env::var("WILAYAH_API_BASE").unwrap_or_else(|_| "https://api.wilayah.workers.dev".to_string())
}

fn fetch_json(url: &str) -> Result<serde_json::Value, CuacaError> {
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| CuacaError::Location(format!("reqwest build error: {}", e)))?;

    if let Err(msg) = BREAKER.before_request() {
        return Err(CuacaError::Location(msg.to_string()));
    }

    let mut attempts = 0;
    loop {
        let response = client.get(url).send();
        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let json = resp
                        .json()
                        .map_err(|e| CuacaError::Location(format!("JSON parse error: {}", e)))?;
                    BREAKER.record_success();
                    return Ok(json);
                } else if status == 429 || status.is_server_error() {
                    BREAKER.record_failure();
                    if attempts < MAX_RETRIES {
                        let backoff_ms = 1000 * (2u64.pow(attempts as u32));
                        std::thread::sleep(Duration::from_millis(backoff_ms));
                        attempts += 1;
                        continue;
                    } else {
                        return Err(CuacaError::Location(format!(
                            "API error after {} retries: {}",
                            MAX_RETRIES, status
                        )));
                    }
                } else {
                    return Err(CuacaError::Location(format!(
                        "API returned error: {}",
                        status
                    )));
                }
            }
            Err(e) => {
                BREAKER.record_failure();
                if attempts < MAX_RETRIES {
                    let backoff_ms = 1000 * (2u64.pow(attempts as u32));
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                    attempts += 1;
                    continue;
                } else {
                    return Err(CuacaError::Location(format!(
                        "network error after {} retries: {}",
                        MAX_RETRIES, e
                    )));
                }
            }
        }
    }
}

/// Lookup nearest village by coordinates. Returns the adm4 code.
pub fn fetch_nearest(lat: f64, lon: f64) -> Result<String, CuacaError> {
    let base = get_base_url();
    let url = format!("{}/nearest?lat={}&lon={}&limit=1", base, lat, lon);
    let json = fetch_json(&url)?;
    if let Some(code) = json["results"].get(0).and_then(|v| v["code"].as_str()) {
        Ok(code.to_string())
    } else {
        Err(CuacaError::Location(
            "no village found for coordinates".to_string(),
        ))
    }
}

/// Search village by name (substring). Returns the first match's adm4 code.
pub fn fetch_search(name: &str) -> Result<String, CuacaError> {
    let base = get_base_url();
    let url = format!("{}/search?q={}&limit=10", base, name);
    let json = fetch_json(&url)?;
    if let Some(code) = json["results"].get(0).and_then(|v| v["code"].as_str()) {
        Ok(code.to_string())
    } else {
        Err(CuacaError::Location(format!(
            "no village found matching '{}'",
            name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use serial_test::serial;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_breaker_initial_state() {
        assert!(BREAKER.before_request().is_ok());
    }

    #[test]
    #[serial]
    fn test_circuit_breaker_transitions() {
        let breaker = CircuitBreaker::new();
        assert!(breaker.before_request().is_ok());
        for _ in 0..BREAKER_THRESHOLD {
            breaker.record_failure();
        }
        assert_eq!(breaker.state.load(Ordering::Relaxed), 1);
        assert!(breaker.before_request().is_err());

        let old = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(BREAKER_COOLDOWN.as_secs() + 1);
        breaker.last_trip.store(old, Ordering::Relaxed);
        assert!(breaker.before_request().is_ok());
        assert_eq!(breaker.state.load(Ordering::Relaxed), 2);
        breaker.record_success();
        assert_eq!(breaker.state.load(Ordering::Relaxed), 0);
    }

    #[test]
    #[serial]
    fn test_circuit_breaker_half_open_failure_reopens() {
        let breaker = CircuitBreaker::new();
        for _ in 0..BREAKER_THRESHOLD {
            breaker.record_failure();
        }
        let old = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(BREAKER_COOLDOWN.as_secs() + 1);
        breaker.last_trip.store(old, Ordering::Relaxed);
        assert!(breaker.before_request().is_ok());
        assert_eq!(breaker.state.load(Ordering::Relaxed), 2);
        breaker.record_failure();
        assert_eq!(breaker.state.load(Ordering::Relaxed), 1);
    }

    #[test]
    #[serial]
    fn test_fetch_nearest_success() {
        reset_breaker();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/nearest?lat=-6.1647&lon=106.8453&limit=1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[{"code":"31.71.03.1001"}]}"#)
            .create();

        std::env::set_var("WILAYAH_API_BASE", &server.url());
        let result = fetch_nearest(-6.1647, 106.8453);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "31.71.03.1001");
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_fetch_nearest_empty() {
        reset_breaker();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/nearest?lat=1.23&lon=4.56&limit=1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[]}"#)
            .create();

        std::env::set_var("WILAYAH_API_BASE", &server.url());
        let result = fetch_nearest(1.23, 4.56);
        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_fetch_search_success() {
        reset_breaker();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/search?q=kemayoran&limit=10")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[{"code":"31.71.03.1001"}]}"#)
            .create();

        std::env::set_var("WILAYAH_API_BASE", &server.url());
        let result = fetch_search("kemayoran");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "31.71.03.1001");
        mock.assert();
    }
}
