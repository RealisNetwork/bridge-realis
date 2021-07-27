use realis_sender::RealisSender;
use bsc_adapter::BSCAdapter;

pub async fn run() {
    let sender = RealisSender::new("rpc.realis.network");

    let adapter =
        BSCAdapter::new("wss://data-seed-prebsc-1-s1.binance.org:8545/", sender)
            .await;

    adapter.listen().await;
}