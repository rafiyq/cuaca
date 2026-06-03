use crate::core::cache::cache_dir;
use crate::core::error::CuacaError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use wilayah;

#[derive(Deserialize, Clone, Debug)]
pub struct GpsCache {
    pub adm4: String,
    pub lat: f64,
    pub lon: f64,
    pub epoch_secs: u64,
}

impl GpsCache {
    pub fn save(&self, path: &PathBuf) -> Result<(), CuacaError> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let mut f = File::create(path)?;
        let content = serde_json::to_string_pretty(&json!({
            "adm4": self.adm4,
            "lat": self.lat,
            "lon": self.lon,
            "epoch_secs": self.epoch_secs
        }))?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() > self.epoch_secs + max_age_secs)
            .unwrap_or(true)
    }
}

pub fn resolve(
    adm4: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
    name: Option<&str>,
) -> Result<String, CuacaError> {
    const GPS_CACHE_MAX_AGE_SECS: u64 = 86400;

    if let Some(code) = adm4 {
        return Ok(code.to_string());
    }

    if lat.is_some() && lon.is_none() {
        return Err(CuacaError::Config("--lat requires --lon".to_string()));
    }
    if lon.is_some() && lat.is_none() {
        return Err(CuacaError::Config("--lon requires --lat".to_string()));
    }

    if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
        let gps_cache_file = cache_dir().join("cuaca-gps.json");
        let cache_valid = read_to_string(&gps_cache_file)
            .ok()
            .and_then(|s| serde_json::from_str::<GpsCache>(&s).ok())
            .and_then(|c| {
                if c.lat == lat_val && c.lon == lon_val && !c.is_stale(GPS_CACHE_MAX_AGE_SECS) {
                    Some(c.adm4)
                } else {
                    None
                }
            });

        if let Some(adm4_code) = cache_valid {
            return Ok(adm4_code);
        }

        let conn = wilayah::open()
            .map_err(|e| CuacaError::Location(format!("failed to open location db: {}", e)))?;

        let results = wilayah::find_nearest(&conn, lat_val, lon_val, 1)
            .map_err(|e| CuacaError::Location(format!("location lookup failed: {}", e)))?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| CuacaError::Unknown(e.to_string()))?
            .as_secs();

        if let Some(village) = results.into_iter().next() {
            let cache = GpsCache {
                adm4: village.code.clone(),
                lat: lat_val,
                lon: lon_val,
                epoch_secs: now,
            };
            // best effort save
            let _ = cache.save(&gps_cache_file);
            return Ok(village.code);
        }

        Err(CuacaError::Location(
            "no village found for coordinates".to_string(),
        ))
    } else if let Some(name_val) = name {
        let conn = wilayah::open()
            .map_err(|e| CuacaError::Location(format!("failed to open location db: {}", e)))?;

        let results = wilayah::find_by_name(&conn, name_val, 10)
            .map_err(|e| CuacaError::Location(format!("name lookup failed: {}", e)))?;

        if results.is_empty() {
            Err(CuacaError::Location(format!(
                "no village found matching '{}'",
                name_val
            )))
        } else {
            Ok(results[0].code.clone())
        }
    } else {
        Err(CuacaError::Location(
            "provide --adm4, --lat/--lon, or --name".to_string(),
        ))
    }
}
