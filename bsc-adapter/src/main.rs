use web3::Web3;
use web3::transports::WebSocket;
use web3::contract::Contract;
use web3::types::{FilterBuilder, U256};
use std::str::FromStr;
use web3::contract::tokens::Detokenize;
use ethabi::{Error, Uint, Address, Bytes};
use logger::logger::{log, Type};
use hex_literal::hex;
use web3::signing::Key;
use ethabi::ethereum_types::H160;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {

    // let url = "wss://bsc-ws-node.nariox.org:443";
    let url = "wss://data-seed-prebsc-1-s1.binance.org:8545/";
    let mut wss = WebSocket::new(url).await;
    loop {
        match wss {
            Ok(_) => break,
            Err(error) => {
                log(Type::Error, String::from("Cannot connect"), &error);
                log(Type::Info, String::from("Try to reconnect"), &());
                wss = WebSocket::new(url).await;
            }
        }
    }
    let web3 = web3::Web3::new(wss.unwrap());
    let json_abi = include_bytes!("../../bsc-sender/res/BEP20.abi");
    let address: web3::types::H160 = web3::types::H160::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef").unwrap();
    let contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
    let from: Address = Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();

    loop {
        // let result: Result<Vec<(Address, Address, Uint)>, Error> =
        //     events(&web3, &contract, "Transfer").await;

        let result: web3::contract::Result<Vec<(Address, Bytes, Uint)>> = contract
            .events(
                "TransferToRealis",
                (),
                (),
                (),
            )
            .await;

        // result.unwrap();
        match result {
            Ok(value) => {
                log(Type::Success, String::from("Got events"), &value);
                let first = value.get(0);
                match first {
                    None => {}
                    Some((from, to, value)) => {
                        log(Type::Info, String::from(""), from);
                        log(Type::Info, String::from(""), to);
                        log(Type::Info, String::from(""), value);
                    }
                }
            }
            Err(error) => log(Type::Error, String::from("Shit happens"), &error)
        }

        sleep(Duration::from_millis(2000)).await;
    }
}