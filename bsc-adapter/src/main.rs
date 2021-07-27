use web3::Web3;
use web3::transports::WebSocket;
use web3::contract::Contract;
use web3::types::{FilterBuilder, U256, U128};
use std::str::FromStr;
use web3::contract::tokens::Detokenize;
use ethabi::{Error, Uint, Address, Bytes};
use logger::logger::{log, Type};
use hex_literal::hex;
use web3::signing::Key;
use ethabi::ethereum_types::H160;
use tokio::time::{sleep, Duration};
use std::sync::mpsc::{channel, Receiver, Sender};
use sp_runtime::AccountId32;
use runtime::AccountId;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct BscAdapter<T: ContractEvents> {
    events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T,
}

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
                // Skip if no events found
                if value.len() == 0 {
                    continue;
                }
                //
                log(Type::Success, String::from("Got events"), &value);
                // Process all events
                for event in value {
                    // Unpack event arguments
                    let (from, to, value) = &event;
                    // Convert argument
                    let account_id = AccountId32::new(<[u8; 32]>::try_from(to.as_slice()).unwrap());
                    // Log arguments
                    log(Type::Success, String::from("Event"), &event);
                    log(Type::Info, String::from("From: "), from);
                    log(Type::Info, String::from("To: "), &account_id);
                    log(Type::Info, String::from("Value: "), value);
                }
            }
            Err(error) => log(Type::Error, String::from("Shit happens"), &error)
        }

        sleep(Duration::from_millis(2500)).await;
    }
}

#[async_trait]
pub trait ContractEvents {
    async fn on_transfer_token_to_realis<'a>(to: AccountId32, value: &u128);
    // async fn on_transfer_nft_to_bsc<'a>(&self, to: &H160, token_id: &TokenId);
}