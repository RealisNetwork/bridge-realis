use log::{debug, error};
use codec::Decode;
use std::sync::mpsc::{channel, Receiver};
use sp_core::sr25519;
use web3::Web3;
use web3::transports::WebSocket;
use web3::futures::{future, StreamExt};
use web3::contract::{Contract, Options};
use web3::types::{Address, TransactionRequest, FilterBuilder};
use std::str::FromStr;
use web3::contract::tokens::{Tokenizable, Detokenize, Tokenize};
use std::{time, fs};
use secp256k1::SecretKey;
use std::path::Path;
use hex_literal::hex;
use web3::types::{H160, H256, U256};
use ethabi::Contract as ethContract;

#[tokio::main]
async fn main() -> web3::Result<()> {
    let _ =env_logger::try_init();
    let wss = web3::transports::WebSocket::new("wss://bsc-ws-node.nariox.org:443").await?;
    let web3 = web3::Web3::new(wss);
    let json_abi = include_bytes!("../res/BEP20.abi");
    let address: web3::types::H160 = web3::types::H160::from_str("0x0db8499bb62772e805af78fc918ee8c8cd6a2859").unwrap();
    let contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
    let from: Address = Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();
    // println!("{:?}", from);

    let a: web3::contract::Result<Vec<u8>> = contract.events("Transfer",
        vec![hex!(
             "78d8af3b0529fcbf811085c11d77397246827610c4f2840fcd551f131644bd3a"
        )],
        vec![hex!(
             "78d8af3b0529fcbf811085c11d77397246827610c4f2840fcd551f131644bd3a"
        )],
        vec![hex!(
             "78d8af3b0529fcbf811085c11d77397246827610c4f2840fcd551f131644bd3a"
        )]).await;
    println!("{:?}", a);

    // let logs = ethContract::event(&contract, "Transfer");

    let filter = FilterBuilder::default()
        .address(vec![address])
        .topics(
            Some(vec![hex!(
                "78d8af3b0529fcbf811085c11d77397246827610c4f2840fcd551f131644bd3a"
            )
                .into()]),
            None,
            None,
            None,
        )
        .build();

    let filter = web3.eth_filter().create_logs_filter(filter).await?;

    let logs_stream = filter.stream(time::Duration::from_secs(1));
    futures::pin_mut!(logs_stream);

    let log = logs_stream.next().await.unwrap();
    println!("got log: {:?}", log);
    Ok(())
}


pub fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
    let string = fs::read_to_string(path).unwrap();
    SecretKey::from_str(&string).unwrap()
}