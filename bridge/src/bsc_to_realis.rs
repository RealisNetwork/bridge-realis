use bsc_adapter::BSCAdapter;
use futures::join;
use realis_sender::RealisSender;

pub async fn run() {
    let sender = RealisSender::new("rpc.realis.network");

    let adapter = BSCAdapter::new(
        "wss://data-seed-prebsc-1-s1.binance.org:8545/",
        sender.clone(),
    )
    .await;
    let token = adapter.listen();

    let adapter_nft = BSCAdapter::new_nft(
        "wss://data-seed-prebsc-1-s1.binance.org:8545/",
        sender,
    )
    .await;
    let nft = adapter_nft.listen_nft();

    join!(token, nft);
}
