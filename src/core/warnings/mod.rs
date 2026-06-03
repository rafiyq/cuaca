//! Weather warnings (nowcast) fetching and parsing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod cache;
mod cap;
mod fetch;
mod polygon;
mod rss;

pub use fetch::fetch_warnings;

/// A weather warning with optional polygon geometry and validity times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub headline: String,
    pub area_desc: String,
    pub effective: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>,
    pub polygons: Vec<Vec<(f64, f64)>>,
    pub web: Option<String>,
}

#[cfg(test)]
mod tests;
