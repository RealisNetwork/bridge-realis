use async_trait::async_trait;
use ethabi::{Address, Bytes, Uint};
use logger::logger::{log, Type};
use sp_runtime::AccountId32;
use std::{convert::TryFrom, str::FromStr};
use tokio::time::{sleep, Duration};
use web3::{
    contract::Contract,
    transports::WebSocket,
    types::{H160, U256},
};

pub struct BSCAdapter<T: ContractEvents> {
    contract: Contract<WebSocket>,
    event_handler: T,
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
        let address: web3::types::H160 = web3::types::H160::from_str(
            "0x987893D34052C07F5959d7e200E9e10fdAf544Ef",
        )
        .unwrap();
        let contract =
            Contract::from_json(web3.eth(), address, json_abi).unwrap();

        BSCAdapter {
            contract,
            event_handler,
        }
    }

    pub async fn listen(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Address, Bytes, Uint)>> =
                self.contract.events("TransferToRealis", (), (), ()).await;

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
                        let account_id = AccountId32::new(
                            <[u8; 32]>::try_from(to.as_slice()).unwrap(),
                        );
                        // Log arguments
                        // log(Type::Info, String::from("From: "), from);
                        // log(Type::Info, String::from("To: "), &account_id);
                        // log(Type::Info, String::from("Value: "), value);
                        //
                        self.event_handler
                            .on_transfer_token_to_realis(
                                account_id,
                                &value.as_u128(),
                            )
                            .await;
                    }
                }
                Err(error) => {
                    log(Type::Error, String::from("Shit happens"), &error)
                }
            }
            // Sleep to do not catch same event twice (2100 - magic number)
            sleep(Duration::from_millis(2050)).await;
        }
    }

    pub async fn new_nft(url: &str, event_handler: T) -> Self {
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

        let json_abi = include_bytes!("../../bsc-sender/res/BEP721.abi");
        // TODO take out into file
        let address: web3::types::H160 = web3::types::H160::from_str(
            "0x81460c30427ee260E06FAecFa17429F56f65423e",
        )
        .unwrap();
        let contract =
            Contract::from_json(web3.eth(), address, json_abi).unwrap();

        BSCAdapter {
            contract,
            event_handler,
        }
    }

    pub async fn listen_nft(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Bytes, Uint, u8)>> = self
                .contract
                .events("TransferNftToRealis", (), (), ())
                .await;

            // result.unwrap();
            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        log(Type::Success, String::from("Event"), &event);
                        // Unpack event arguments
                        let (to, value, basic) = &event;
                        // Convert argument
                        let account_id = AccountId32::new(
                            <[u8; 32]>::try_from(to.as_slice()).unwrap(),
                        );
                        // Log arguments
                        // log(Type::Info, String::from("From: "), from);
                        log(Type::Info, String::from("To: "), &account_id);
                        log(Type::Info, String::from("Value: "), value);
                        log(Type::Info, String::from("Basic: "), basic);
                        //
                        self.event_handler
                            .on_transfer_nft_to_realis(
                                account_id, &value, *basic,
                            )
                            .await;
                    }
                }
                Err(error) => {
                    log(Type::Error, String::from("Event error"), &error)
                }
            }
            // Sleep to do not catch same event twice (2100 - magic number)
            sleep(Duration::from_millis(2100)).await;
        }
    }
}

#[async_trait]
pub trait ContractEvents {
    async fn on_transfer_token_to_realis<'a>(
        &self,
        to: AccountId32,
        value: &u128,
    );
    async fn on_transfer_nft_to_realis<'a>(
        &self,
        to: AccountId32,
        token_id: &U256,
        basic: u8,
    );
}
