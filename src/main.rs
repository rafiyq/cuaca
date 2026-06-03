use clap::Parser;
use cuaca::cli::args::Args;
use cuaca::cli::run::run;
use cuaca::util::error_json;

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        println!("{}", error_json("⛔️", &e.to_string()));
        std::process::exit(1);
    }
}
