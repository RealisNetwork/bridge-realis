use std::env;

use logger::prelude::*;

mod bsc_to_realis;
mod logger;
mod realis_to_bsc;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "INFO"); // TODO: Pass log level via conf.

    let logger = logger::new(std::io::stdout(), std::io::stderr());
    let _scope_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    // Get command lines arguments
    let args: Vec<String> = env::args().collect();
    // Get command line first argument
    let arg = args.get(1);

    match arg {
        None => error!("Specify flag (realis-to-bsc or bsc-to-realis)"),
        Some(value) => match value.as_str() {
            "realis-to-bsc" => realis_to_bsc::run().await,
            "bsc-to-realis" => bsc_to_realis::run().await,
            _ => error!("Unknown command!"),
        },
    }
}
