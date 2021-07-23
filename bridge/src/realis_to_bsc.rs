use realis_adapter::*;

pub fn run() {
    // Init realis part of relay
    let adapter = RealisAdapter::new(String::from("rpc.realis.network"));
    // Init bsc part of relay
    // let sender = BscSender::new(String::from());
    // Start listening for events
    adapter.listener();
}