use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

/// Returns the base cache directory for cuaca.
/// Priority:
/// 1. `CUACA_CACHE_DIR` environment variable if set
/// 2. Platform-appropriate user cache directory via `directories` crate
/// 3. Fallback to temporary directory (`$TEMP/cuaca`)
pub fn cache_dir() -> PathBuf {
    if let Ok(path) = env::var("CUACA_CACHE_DIR") {
        return PathBuf::from(path);
    }
    if let Some(proj_dirs) = ProjectDirs::from("", "cuaca", "cuaca") {
        return proj_dirs.cache_dir().to_path_buf();
    }
    env::temp_dir().join("cuaca")
}

#[cfg(test)]
mod tests {
    use super::*;
    use directories::ProjectDirs;

    #[test]
    fn test_cache_dir() {
        // Save previous state
        let prev = env::var("CUACA_CACHE_DIR").ok();

        // When env set
        env::set_var("CUACA_CACHE_DIR", "/tmp/custom-cache");
        assert_eq!(cache_dir(), PathBuf::from("/tmp/custom-cache"));

        // When env not set (default)
        env::remove_var("CUACA_CACHE_DIR");
        let expected = if let Some(proj_dirs) = ProjectDirs::from("", "cuaca", "cuaca") {
            proj_dirs.cache_dir().to_path_buf()
        } else {
            env::temp_dir().join("cuaca")
        };
        assert_eq!(cache_dir(), expected);

        // Restore previous state
        match prev {
            Some(val) => env::set_var("CUACA_CACHE_DIR", val),
            None => env::remove_var("CUACA_CACHE_DIR"),
        }
    }
}
