use clap::Parser;
use cuaca::util::error_json;
use cuaca_cli::cli::args::{Command, Root};
use cuaca_cli::stats;
#[cfg(unix)]
use cuaca_cli::{client, server};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::parse();

    match root.cmd {
        None => {
            // Direct mode
            if let Err(e) = cuaca_cli::cli::run::run(root.args) {
                println!("{}", error_json("⛔️", &e.to_string()));
                std::process::exit(1);
            }
        }
        #[cfg(unix)]
        Some(Command::Server(opts)) => {
            if let Err(e) = server::start(opts.archive, opts.socket, opts.ttl) {
                eprintln!("server error: {}", e);
                std::process::exit(1);
            }
        }
        #[cfg(unix)]
        Some(Command::Client { raw }) => {
            if let Err(e) = client::send_request(&root.args, raw) {
                eprintln!("client error: {}", e);
                std::process::exit(1);
            }
        }
        #[cfg(not(unix))]
        Some(Command::Server(_)) | Some(Command::Client { .. }) => {
            eprintln!("server/client mode is only supported on Unix");
            std::process::exit(1);
        }
        Some(Command::Stats(opts)) => {
            if let Err(e) = stats::compute_and_print(opts) {
                eprintln!("stats error: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
