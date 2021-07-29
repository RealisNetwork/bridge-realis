use bsc_adapter::BSCAdapter;
use realis_sender::RealisSender;

pub async fn run() {
    let sender = RealisSender::new("rpc.realis.network");

    let adapter = BSCAdapter::new(
        "wss://data-seed-prebsc-1-s1.binance.org:8545/",
        sender.clone(),
    )
    .await;
    adapter.listen().await;

    // let adapter_nft = BSCAdapter::new_nft(
    //     "wss://data-seed-prebsc-1-s1.binance.org:8545/",
    //     sender,
    // )
    // .await;
    // adapter_nft.listen_nft().await;
}
