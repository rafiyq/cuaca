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
    fn test_cache_dir_uses_env() {
        env::set_var("CUACA_CACHE_DIR", "/tmp/custom-cache");
        let dir = cache_dir();
        assert_eq!(dir, PathBuf::from("/tmp/custom-cache"));
        env::remove_var("CUACA_CACHE_DIR");
    }

    #[test]
    fn test_cache_dir_default() {
        // Ensure env var is not set
        env::remove_var("CUACA_CACHE_DIR");
        let dir = cache_dir();
        // Should be temp_dir() / "cuaca"
        assert_eq!(dir, env::temp_dir().join("cuaca"));
    }
}
