use clap::Parser;
use telemetry_server::analytics::{self, Cli};

fn main() {
    if let Err(e) = analytics::run(Cli::parse()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
