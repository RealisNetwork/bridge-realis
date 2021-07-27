use realis_sender::RealisSender;
use bsc_adapter::BSCAdapter;

pub async fn run() {
    let sender = RealisSender::new("rpc.realis.network");

    // let adapter =
    //     BSCAdapter::new("wss://data-seed-prebsc-1-s1.binance.org:8545/", sender.clone())
    //         .await;

    let adapter_nft =
        BSCAdapter::new_nft("wss://data-seed-prebsc-1-s1.binance.org:8545/", sender)
            .await;

    adapter_nft.listen_nft().await;
    // adapter.listen().await;
}