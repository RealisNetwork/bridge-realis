use log::{debug, error};
use codec::Decode;
use std::sync::mpsc::{channel, Receiver};
use sp_core::sr25519;
use web3::Web3;
use web3::transports::WebSocket;
use web3::futures::{future, StreamExt};
use web3::contract::{Contract, Options};
use std::time;
use web3::types::{Address, CallRequest, U256};
use primitive_types::H160;
use std::str::FromStr;
use web3::ethabi::Token;
use web3::contract::tokens::Tokenizable;

#[tokio::main]
async fn main() -> web3::Result<()> {
    let _ =env_logger::try_init();
    let wss = web3::transports::WebSocket::new("wss://data-seed-prebsc-1-s1.binance.org:8545/").await?;
    let web3 = web3::Web3::new(wss);
    let mut sub = web3.eth_subscribe().subscribe_new_heads().await?;

    println!("Got subscription id: {:?}", sub.id());


    // (&mut sub)
    //     .take(5)
    //     .for_each(|x| {
    //         println!("Block number: {:?}", x.unwrap().number.unwrap());
    //         future::ready(())
    //     })
    //     .await;

    let json_abi = include_bytes!("../res/BEP20.abi");

    let address: Address = Address::from_str("0x49cd5f0b0314aa2884905cf3c031c18ea36b5c9d").unwrap();
    println!("ping!");
    let registry_contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
    let from: Address = Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();
    let to: Address = Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap();
    let value = U256([1, 0, 0, 0 as u64]);
    // let params = Tokenizable::into_token([from, to]);
    let params = (to, value, );

    let result: web3::contract::Result<bool> = registry_contract
        .query("transfer", params, from, Options::default(), None)
        .await;
    println!("{:?}", result);
    Ok(())
}
