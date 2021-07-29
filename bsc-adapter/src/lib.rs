use async_trait::async_trait;
use ethabi::{Address, Bytes, Uint};
use log::{error, info};
use realis_primitives::TokenId;
use runtime::AccountId;
use sp_core::Decode;
use utils::contract;
use web3::{contract::Contract, transports::WebSocket};
use futures::join;

pub struct BSCAdapter<T: ContractEvents> {
    token_contract: Contract<WebSocket>,
    nft_contract: Contract<WebSocket>,
    event_handler: T,
}

impl<T: ContractEvents> BSCAdapter<T> {
    /// # Panics
    ///
    /// Conection to BSC for transfer tokens
    pub async fn new(url: &str, event_handler: T) -> Self {
        let token_contract = contract::token_new(url).await;
        let nft_contract = contract::nft_new(url).await;

        BSCAdapter {
            token_contract,
            nft_contract,
            event_handler,
        }
    }

    pub async fn listen(&self) {
        let token = self.listen_token();
        let nft = self.listen_nft();

        join!(token, nft);
    }

    async fn listen_token(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Address, Bytes, Uint)>> =
                self.token_contract.events("TransferToRealis", (), (), ()).await;

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
        }
    }

    async fn listen_nft(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Bytes, Uint, u8)>> = self
                .nft_contract
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
