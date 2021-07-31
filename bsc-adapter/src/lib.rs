use ethabi::{Address, Bytes, Uint};
use futures::join;
use log::{error, info};
use realis_primitives::TokenId;
use runtime::AccountId;
use sp_core::Decode;
use sp_core::H160;
use utils::contract;
use web3::{contract::Contract, transports::WebSocket};
use realis_sender::RealisSender;

// TODO from struct to functions??? or find better solution
struct BSCListener {
    contract: Contract<WebSocket>,
}

impl BSCListener {
    pub fn new(contract: Contract<WebSocket>) -> Self {
        BSCListener{
            contract,
        }
    }

    pub async fn listen_token(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Address, Bytes, Uint)>> = self.contract.events("TransferToRealis", (), (), ()).await;
            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (from, to, value) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &to[..]).unwrap_or_default();
                        // Log arguments
                        println!(
                            "TransferTokenToRealis: {:?} => {:?}, {:?}",
                            from, to, value
                        );
                        RealisSender::send_token_to_realis(H160::from(from.0), account_id, value.as_u128()).await;
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_nft(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Address, Bytes, Uint, u8)>> =
                self.contract.events("TransferNftToRealis", (), (), ()).await;

            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (from, to, token_id, basic) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &to[..]).unwrap_or_default();
                        // Log arguments
                        println!(
                            "TransferNftToRealis: {:?}, {:?}, {:?}",
                            to, token_id, basic
                        );
                        RealisSender::send_nft_to_realis(H160::from(from.0), account_id, *token_id, *basic).await;
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_token_success(&self) {
        // loop {
        //     let logs: web3::contract::Result<Vec<(Bytes, Address, Uint)>> = self.contract.events("Transfer", (), (), ()).await;
        //     match logs {
        //         Ok(events) => {
        //             // Process all events
        //             for event in events {
        //                 // Log event
        //                 println!("Get event {:?}", event);
        //                 // Unpack event arguments
        //                 let (from, to, amount) = &event;
        //                 // Convert argument
        //                 let account_id =
        //                     AccountId::decode(&mut &from[..]).unwrap_or_default();
        //                 // Log arguments
        //                 println!(
        //                     "TokenSuccessOnBsc: {:?} => {:?}, {:?}",
        //                     account_id, to, amount
        //                 );
        //                 RealisSender::send_token_approve_to_realis(account_id, amount.as_u128()).await;
        //             }
        //         }
        //         Err(error) => {
        //             println!("Error while listen {:?}", error);
        //         }
        //     }
        // }
    }

    pub async fn listen_nft_success(&self) {
        // loop {
        //     let logs: web3::contract::Result<Vec<(Bytes, Address, Uint, u8)>> =
        //         self.contract.events("MintNftFromRealis", (), (), ()).await;
        //
        //     match logs {
        //         Ok(events) => {
        //             // Process all events
        //             for event in events {
        //                 // Log event
        //                 println!("Get event {:?}", event);
        //                 // Unpack event arguments
        //                 let (from, to, token_id, basic) = &event;
        //                 // Convert argument
        //                 let account_id =
        //                     AccountId::decode(&mut &from[..]).unwrap_or_default();
        //                 // Log arguments
        //                 println!(
        //                     "TransferNftToRealis: {:?} => {:?}, {:?}, {:?}",
        //                     account_id, to, token_id, basic
        //                 );
        //                 RealisSender::send_nft_approve_to_realis(account_id, *token_id).await;
        //             }
        //         }
        //         Err(error) => {
        //             println!("Error while listen {:?}", error);
        //         }
        //     }
        // }
    }
}

pub struct BSCAdapter {}

impl BSCAdapter {
    pub async fn listen() {
        let token_listener = BSCListener::new(
            contract::token_new().await,
        );
        let nft_listener = BSCListener::new(
            contract::nft_new().await,
        );
        let token_listener_success = BSCListener::new(
            contract::token_new().await,
        );
        let nft_listener_success = BSCListener::new(
            contract::nft_new().await,
        );

        join!(
            token_listener.listen_token(),
            nft_listener.listen_nft(),
            token_listener_success.listen_token_success(),
            nft_listener_success.listen_nft_success()
        );
    }
}