use log::{debug, error};
use codec::Decode;
use std::sync::mpsc::{channel, Receiver};
use sp_core::sr25519;
use web3::Web3;
use web3::transports::WebSocket;
use web3::futures::{future, StreamExt};
use web3::contract::{Contract, Options};
use std::time;
use web3::types::{Address, CallRequest, U256, H256, H160, TransactionParameters, TransactionRequest};
use web3::api::Eth;
use std::str::FromStr;
use web3::ethabi::Token;
use hex_literal::hex;
use secp256k1::SecretKey;
use web3::contract::tokens::Tokenize;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
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

    let address: Address  = hex!("0db8499bb62772e805af78fc918ee8c8cd6a2859").into();
    println!("ping!");
    let registry_contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
    let value = U256::from(10_000u64);

    let from: Address = Address::from_str("6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();
    let to: Address = Address::from_str("12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap();

    let params = (to, value);
    println!("{}", value);
    println!("{}", from);
    println!("{:?}", params);
    // let accounts = registry_contract.eth().accounts().await?;
    // println!("{:?}", accounts);

    let tx_params = TransactionRequest {
        from,
        to: Some(to),
        gas: None,
        gas_price: None,
        value: Some(value),
        data: None,
        nonce: None,
        condition: None,
        transaction_type: None,
        access_list: None
    };

    // let params = Tokenize::into_tokens(tx_params);

    let result = registry_contract
        .call("transfer", (to,  value), from, Options::default()).await;
    println!("{:?}", result);
    Ok(())
}
