use async_trait::async_trait;
use ethabi::{Address, Bytes, Uint};
use log::{error, info};
use realis_primitives::TokenId;
use runtime::AccountId;
use sp_core::Decode;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use web3::{contract::Contract, transports::WebSocket};

pub struct BSCAdapter<T: ContractEvents> {
    contract: Contract<WebSocket>,
    event_handler: T,
}

impl<T: ContractEvents> BSCAdapter<T> {
    /// # Panics
    ///
    /// Conection to BSC for transfer tokens
    pub async fn new(url: &str, event_handler: T) -> Self {
        // TODO take out in separate function
        // Connect to bsc by web socket
        let mut wss = WebSocket::new(url).await;
        // Try connect again if connection fail
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!("Performing reconnect. Cause: {:?}", error);
                    wss = WebSocket::new(url).await;
                }
            }
        }
        let web3 = web3::Web3::new(wss.unwrap());

        let json_abi = include_bytes!("../../utils/res/BEP20.abi");
        // TODO take out into file
        let address: web3::types::H160 = web3::types::H160::from_str(
            "0x30a02a714Ea7674F1988ED5d81094F775b28E611",
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
                        info!("Get event {:?}", event);
                        // Unpack event arguments
                        let (from, to, value) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &to[..]).unwrap_or_default();
                        // Log arguments
                        info!(
                            "TransferToRealis: {:?} => {:?}, {:?}",
                            from, to, value
                        );

                        self.event_handler
                            .on_transfer_token_to_realis(
                                account_id,
                                &value.as_u128(),
                            )
                            .await;
                    }
                }
                Err(error) => error!("Error while listen {:?}", error),
            }
            // Sleep to do not catch same event twice (2100 - magic number)
            sleep(Duration::from_millis(2050)).await;
        }
    }

    /// # Panics
    ///
    /// Conection to BSC for transfer NFT
    pub async fn new_nft(url: &str, event_handler: T) -> Self {
        // TODO take out in separate function
        // Connect to bsc by web socket
        let mut wss = WebSocket::new(url).await;
        // Try connect again if connection fail
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!("Performing reconnect. Cause: {:?}", error);
                    wss = WebSocket::new(url).await;
                }
            }
        }
        let web3 = web3::Web3::new(wss.unwrap());

        let json_abi = include_bytes!("../../utils/res/BEP721.abi");
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

            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        info!("Get event {:?}", event);
                        // Unpack event arguments
                        let (to, value, basic) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &to[..]).unwrap_or_default();
                        // Log arguments
                        info!(
                            "TransferNftToRealis: {:?}, {:?}, {:?}",
                            to, value, basic
                        );

                        self.event_handler
                            .on_transfer_nft_to_realis(
                                account_id,
                                value.into(),
                                *basic,
                            )
                            .await;
                    }
                }
                Err(error) => error!("Error while listen {:?}", error),
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
        to: AccountId,
        value: &u128,
    );
    async fn on_transfer_nft_to_realis<'a>(
        &self,
        to: AccountId,
        token_id: TokenId,
        basic: u8,
    );
}
