use crate::cli::args::Args;
use crate::cli::run::run_internal;
use crate::core::error::CuacaError;
use crate::core::weather::ensure_forecast;
use serde_json;
use std::env;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use signal_hook::consts::signal::*;
#[cfg(unix)]
use signal_hook::flag as signal_flag;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

const DEFAULT_SOCKET: &str = "cuaca.sock";

#[derive(Clone)]
struct Daemon {
    archive: bool,
    ttl: Duration,
}

#[cfg(unix)]
impl Daemon {
    fn start(socket: Option<PathBuf>, archive: bool, ttl_secs: u64) -> Result<(), CuacaError> {
        let cache_dir = crate::core::cache::cache_dir();
        let socket_path = socket.unwrap_or_else(|| {
            env::var("CUACA_SOCKET")
                .map(PathBuf::from)
                .unwrap_or_else(|_| cache_dir.join(DEFAULT_SOCKET))
        });

        // Ensure parent dir exists
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove stale socket if exists
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }

        let listener = UnixListener::bind(&socket_path)?;
        // Set socket permissions to 0600
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&socket_path, perms)?;

        // Write PID file
        let pid_path = socket_path.with_extension("pid");
        std::fs::write(&pid_path, format!("{}", std::process::id()))?;

        let running = Arc::new(AtomicBool::new(true));
        // Register signal handlers
        let _ = signal_flag::register(SIGTERM, Arc::clone(&running));
        let _ = signal_flag::register(SIGINT, Arc::clone(&running));

        eprintln!("cuaca server listening on {:?}", socket_path);

        let daemon = Daemon {
            archive,
            ttl: Duration::from_secs(ttl_secs),
        };

        while running.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let daemon = daemon.clone();
                    thread::spawn(move || {
                        let _ = daemon.handle_connection(stream);
                    });
                }
                Err(e) => {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    eprintln!("accept error: {}", e);
                }
            }
        }

        // Cleanup
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&pid_path);
        Ok(())
    }

    fn handle_connection(&self, stream: UnixStream) -> Result<(), CuacaError> {
        // Read a single line request
        let mut reader = std::io::BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            return Ok(());
        }
        let request: Args = serde_json::from_str(line.trim())?;

        // Resolve location
        let adm4 = crate::core::location::resolve(
            request.adm4.as_deref(),
            request.lat,
            request.lon,
            request.name.as_deref(),
        )?;

        // Ensure we have fresh forecast (cache or fetch)
        let forecast = ensure_forecast(&adm4, self.ttl.as_secs(), self.archive)?;

        // Build response
        let output = if request.raw {
            serde_json::to_string_pretty(&forecast)?
        } else {
            run_internal(&request, Some(forecast))?
        };

        // Write response and close connection
        let mut writer = BufWriter::new(stream);
        writer.write_all(format!("{}\n", output).as_bytes())?;
        writer.flush()?;
        Ok(())
    }
}

/// Public entry (works on all platforms; returns unsupported error on non‑Unix)
pub fn start(
    archive: bool,
    socket: Option<PathBuf>,
    ttl_minutes: Option<u64>,
) -> Result<(), CuacaError> {
    #[cfg(unix)]
    {
        let ttl = ttl_minutes.map_or(600, |m| m * 60);
        Daemon::start(socket, archive, ttl)
    }
    #[cfg(not(unix))]
    {
        Err(CuacaError::Config(
            "daemon mode is only supported on Unix".to_string(),
        ))
    }
}
