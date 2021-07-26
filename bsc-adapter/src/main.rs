use web3::Web3;
use web3::transports::WebSocket;
use web3::contract::Contract;
use web3::types::{FilterBuilder, U256};
use std::str::FromStr;
use web3::contract::tokens::Detokenize;
use ethabi::{Error, Uint, Address};
use env_logger;
use secp256k1::SecretKey;
use std::path::Path;
use std::fs;

use hex_literal::hex;

use logger::logger::{log, Type};

#[tokio::main]
async fn main() {
    let _ = env_logger::init();

    let wss = web3::transports::WebSocket::new("wss://bsc-ws-node.nariox.org:443").await.unwrap();
    let web3 = web3::Web3::new(wss);
    let json_abi = include_bytes!("../../bsc-sender/res/BEP20.abi");
    let address: web3::types::H160 = web3::types::H160::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef").unwrap();
    let contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
    let from: Address = Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();

    loop {
        // let result: Result<Vec<(Address, Address, Uint)>, Error> =
        //     events(&web3, &contract, "Transfer").await;

        let result: web3::contract::Result<Vec<(Address, Address)>> = contract
            .events(
                "TransferToRealis",
                (),
                (),
                (),
            )
            .await;

        // result.unwrap();
        match result {
            Ok(value) => log(Type::Info, String::from("Got events"), &value),
            Err(error) => log(Type::Error, String::from("Shit happens"), &error)
        }
    }
}

async fn events<R: Detokenize>(web3: &Web3<WebSocket>, contract: &Contract<WebSocket>, event: &str) -> Result<Vec<R>, Error> {
    let res = contract.abi().event(event).and_then(|ev| {
        let filter = ev.filter(ethabi::RawTopicFilter {
            topic0: ethabi::Topic::Any,
            topic1: ethabi::Topic::Any,
            topic2: ethabi::Topic::Any,
        })?;
        Ok((ev.clone(), filter))
    });

    let (ev, filter) = match res {
        Ok(x) => x,
        Err(e) => return Err(e.into()),
    };

    let logs = web3
        .eth()
        .logs(FilterBuilder::default().topic_filter(filter).build())
        .await
        .unwrap();
    logs.into_iter()
        .map(move |l| {
            let log = ev.parse_log(ethabi::RawLog {
                topics: l.topics,
                data: l.data.0,
            })?;

            Ok(R::from_tokens(
                log.params.into_iter().map(|x| x.value).collect::<Vec<_>>(),
            ).unwrap())
        })
        .collect::<Result<Vec<R>, Error>>()
}