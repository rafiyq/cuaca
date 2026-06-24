use crate::cli::args::Args;
use crate::cli::run::run;
use serde_json;
use std::error::Error;

fn send_request_unix(args: &Args, raw: bool) -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    // Determine socket path
    let socket_path = env::var("CUACA_SOCKET").unwrap_or_else(|_| {
        let cache_dir = crate::core::cache::cache_dir();
        cache_dir.join("cuaca.sock").to_string_lossy().into_owned()
    });

    // Try to connect
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            // Build request (include raw flag)
            let mut request = args.clone();
            request.raw = raw;
            let payload = serde_json::to_string(&request)? + "\n";

            // Send with timeout
            stream.set_write_timeout(Some(Duration::from_secs(2)))?;
            stream.write_all(payload.as_bytes())?;

            // Read response
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            let mut buf = String::new();
            stream.read_to_string(&mut buf)?;
            print!("{}", buf);
        }
        Err(_) => {
            // Daemon not running, fallback to direct fetch
            eprintln!("daemon unavailable, falling back to direct fetch");
            // Use direct mode with same args (including raw)
            run(args.clone())?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn send_request_other(_args: &Args, _raw: bool) -> Result<(), Box<dyn Error>> {
    Err("client mode is only supported on Unix".into())
}

/// Sends request to daemon and prints response. Falls back to direct fetch if daemon unavailable.
pub fn send_request(args: &Args, raw: bool) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    {
        send_request_unix(args, raw)
    }
    #[cfg(not(unix))]
    {
        send_request_other(args, raw)
    }
}
