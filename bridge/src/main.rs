// use logger::prelude::*;
use std::env;

mod logger;

use bsc_adapter::BSCAdapter;
// use bsc_sender::BscSender;
// use futures::join;
// use message_broker;
use realis_adapter::RealisAdapter;
// use realis_sender::RealisSender;

// use futures::executor::block_on;
// use std::{
//     sync::mpsc::{channel, Receiver, RecvError, Sender},
//     thread,
// };

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
        None => println!(
            "Specify flag (realis-to-bsc or bsc-to-realis or message-broker)"
        ),
        Some(value) => match value.as_str() {
            "realis-to-bsc" => {
                let mut realis_adapter = RealisAdapter::new("rpc.realis.network");

                realis_adapter.listen().await;
            }
            "bsc-to-realis" => {
                BSCAdapter::listen().await;
            }
            "message-broker" => {
                message_broker::message_broker().await;
            }
            _ => println!("Unknown command!"),
        },
    }
}
