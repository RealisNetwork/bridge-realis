use std::env;
use logger::prelude::*;

mod logger;

use realis_adapter::RealisAdapter;
use bsc_sender::BscSender;
use bsc_adapter::BSCAdapter;
use realis_sender::RealisSender;
use futures::join;

use std::sync::mpsc::{channel, Receiver, Sender, RecvError};
use std::thread;
use futures::executor::block_on;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "INFO"); // TODO: Pass log level via conf.

    // let logger = logger::new(std::io::stdout(), std::io::stderr());
    let logger = logger::term_new();
    let _scope_guard = slog_scope::set_global_logger(logger);
    // slog_stdlog::init().unwrap();

    // Get command lines arguments
    let args: Vec<String> = env::args().collect();
    // Get command line first argument
    let arg = args.get(1);

    match arg {
        None => println!("Specify flag (realis-to-bsc or bsc-to-realis)"),
        Some(value) => match value.as_str() {
            "realis-to-bsc" => {
                let realis_adapter =
                    RealisAdapter::new(
                        "rpc.realis.network",
                    );

                realis_adapter.listen().await;
            }
            "bsc-to-realis" => {
                BSCAdapter::listen().await;
            }
            _ => println!("Unknown command!"),
        },
    }
}
