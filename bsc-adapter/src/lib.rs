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

pub struct BSCAdapter<T: ContractEvents> {
    contract: Contract<WebSocket>,
    event_handler: T
}

impl<T: ContractEvents> BSCAdapter<T> {
    pub async fn new(url: &str, event_handler: T) -> Self {
        // TODO take out in separate function
        // Connect to bsc by web socket
        let mut wss = WebSocket::new(url).await;
        // Try connect again if connection fail
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
        // TODO take out into file
        let address: web3::types::H160 = web3::types::H160::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef").unwrap();
        let contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();

        BSCAdapter {
            contract,
            event_handler
        }
    }

    pub async fn listen(&self) {
        loop {

            let logs: web3::contract::Result<Vec<(Address, Bytes, Uint)>> = self.contract
                .events(
                    "TransferToRealis",
                    (),
                    (),
                    (),
                )
                .await;

            // result.unwrap();
            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        log(Type::Success, String::from("Event"), &event);
                        // Unpack event arguments
                        let (from, to, value) = &event;
                        // Convert argument
                        let account_id = AccountId32::new(<[u8; 32]>::try_from(to.as_slice()).unwrap());
                        // Log arguments
                        log(Type::Info, String::from("From: "), from);
                        log(Type::Info, String::from("To: "), &account_id);
                        log(Type::Info, String::from("Value: "), value);
                        //
                        self.event_handler.on_transfer_token_to_realis(account_id, &value.as_u128())
                            .await;
                    }
                }
                Err(error) => log(Type::Error, String::from("Shit happens"), &error)
            }
            // Sleep to do not catch same event twice (2500 - magic number)
            sleep(Duration::from_millis(2500)).await;
        }
    }
}

#[async_trait]
pub trait ContractEvents {
    async fn on_transfer_token_to_realis<'a>(&self, to: AccountId32, value: &u128);
    // async fn on_transfer_nft_to_realis<'a>(&self, to: &H160, token_id: &U256);
}