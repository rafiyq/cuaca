use std::env;
use std::path::PathBuf;

/// Returns the base cache directory for cuaca.
/// If `CUACA_CACHE_DIR` is set, uses that; otherwise falls back to `$TEMP/cuaca`.
pub fn cache_dir() -> PathBuf {
    env::var("CUACA_CACHE_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cuaca"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        // Save previous state
        let prev = env::var("CUACA_CACHE_DIR").ok();

        // When env set
        env::set_var("CUACA_CACHE_DIR", "/tmp/custom-cache");
        assert_eq!(cache_dir(), PathBuf::from("/tmp/custom-cache"));

        // When env not set (default)
        env::remove_var("CUACA_CACHE_DIR");
        assert_eq!(cache_dir(), env::temp_dir().join("cuaca"));

        // Restore previous state
        match prev {
            Some(val) => env::set_var("CUACA_CACHE_DIR", val),
            None => env::remove_var("CUACA_CACHE_DIR"),
        }
    }
}
