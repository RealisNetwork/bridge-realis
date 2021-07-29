use bsc_adapter::BSCAdapter;
use realis_sender::RealisSender;

pub async fn run() {
    let sender = RealisSender::new("rpc.realis.network");

    let mut adapter = BSCAdapter::new(
        sender,
    )
    .await;

    adapter.listen().await;
}
