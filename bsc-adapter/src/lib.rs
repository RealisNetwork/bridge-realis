use log::{debug, error};
use codec::Decode;
use std::sync::mpsc::{channel, Receiver};
use sp_core::sr25519;
use web3::Web3;
use web3::transports::WebSocket;
use web3::futures::{future, StreamExt};

#[tokio::main]
async fn main() -> web3::Result<()> {
    let _ =env_logger::try_init();
    println!("0");
    let wss = web3::transports::WebSocket::new("wss://bsc-ws-node.nariox.org:443").await?;
    println!("2");
    let web3 = web3::Web3::new(wss);
    println!("2");
    let mut sub = web3.eth_subscribe().subscribe_new_heads().await?;
    println!("3");

    println!("Got subscription id: {:?}", sub.id());

    (&mut sub)
        .take(5)
        .for_each(|x| {
            println!("Got: {:?}", x);
            future::ready(())
        })
        .await;

    sub.unsubscribe().await?;

    Ok(())
}