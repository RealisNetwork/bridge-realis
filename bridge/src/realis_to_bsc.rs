use realis_adapter::RealisAdapter;
use bsc_sender::BscSender;

pub async fn run() {
    // Init bsc part of relay
    let sender = BscSender::new().await;
    // Init realis part of relay
    let adapter = RealisAdapter::new(String::from("rpc.realis.network"), sender);
    // Start listening for events
    adapter.listener().await;
}