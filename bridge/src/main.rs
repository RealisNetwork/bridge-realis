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


    let (to_bsc_sender, receiver_to_bsc_sender) = channel();
    let (to_realis_sender, receiver_to_realis_sender) = channel();

    let realis_adapter =
        RealisAdapter::new(
            "rpc.realis.network",
            to_bsc_sender.clone(),
            to_realis_sender.clone()
        );
    // let bsc_sender =
    //     BscSender::new(
    //         receiver_to_bsc_sender
    //     ).await;
    // let bsc_adapter =
    //     BSCAdapter::new(
    //         to_bsc_sender.clone(),
    //         to_realis_sender.clone()
    //     );
    // let realis_sender =
    //     RealisSender::new(
    //         receiver_to_realis_sender
    //     );

    realis_adapter.listen().await;


    // let realis_adapter_thread = thread::spawn(move || {
    //     block_on(realis_adapter.listen());
    // });
    // let bsc_sender_thread = thread::spawn(move || {
    //     block_on(bsc_sender.listen());
    // });
    // let bsc_adapter_thread = thread::spawn(move || {
    //     block_on(bsc_adapter.listen());
    // });
    // let realis_sender_thread = thread::spawn(move || {
    //     block_on(realis_sender.listen());
    // });

    // realis_adapter_thread.join().unwrap();
    // bsc_sender_thread.join().unwrap();
    // bsc_adapter_thread.join().unwrap();
    // realis_sender_thread.join().unwrap();
}
