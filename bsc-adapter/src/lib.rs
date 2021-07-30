use ethabi::{Address, Bytes, Uint};
use futures::join;
use log::{error, info};
use realis_primitives::TokenId;
use runtime::AccountId;
use sp_core::Decode;
use sp_core::H160;
use utils::contract;
use web3::{contract::Contract, transports::WebSocket};

use std::sync::mpsc::{channel, Receiver, Sender};
use bridge_events::Events;

// TODO from struct to functions??? or find better solution
struct BSCListener {
    contract: Contract<WebSocket>,
    channel_to_realis_sender: Sender<Events>
}

impl BSCListener {
    pub fn new(contract: Contract<WebSocket>, channel_to_bsc_sender: Sender<Events>) -> Self {
        BSCListener{
            contract,
            channel_to_realis_sender: channel_to_bsc_sender
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
                            "TransferToRealis: {:?} => {:?}, {:?}",
                            from, to, value
                        );
                        self.channel_to_realis_sender.send(Events::TokenBscToRealis(account_id, value.as_u128()));
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
            let logs: web3::contract::Result<Vec<(Bytes, Uint, u8)>> =
                self.contract.events("TransferNftToRealis", (), (), ()).await;

            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (to, value, basic) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &to[..]).unwrap_or_default();
                        // Log arguments
                        println!(
                            "TransferNftToRealis: {:?}, {:?}, {:?}",
                            to, value, basic
                        );
                        self.channel_to_realis_sender.send(Events::NftBcsToRealis(account_id, *value, *basic));
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_token_success(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Bytes, Address, Uint)>> = self.contract.events("Transfer", (), (), ()).await;
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
                            AccountId::decode(&mut &from[..]).unwrap_or_default();
                        // Log arguments
                        println!(
                            "TokenSuccessOnBsc: {:?} => {:?}, {:?}",
                            account_id, to, value
                        );
                        self.channel_to_realis_sender.send(Events::TokenSuccessOnBsc(account_id, value.as_u128()));
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_nft_success(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Bytes, Address, Uint, u8)>> =
                self.contract.events("MintNftFromRealis", (), (), ()).await;

            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (from, to, value, basic) = &event;
                        // Convert argument
                        let account_id =
                            AccountId::decode(&mut &from[..]).unwrap_or_default();
                        // Log arguments
                        println!(
                            "TransferNftToRealis: {:?} => {:?}, {:?}, {:?}",
                            account_id, to, value, basic
                        );
                        self.channel_to_realis_sender.send(Events::NftSuccessOnBsc(account_id, *value, *basic));
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }
}

pub struct BSCAdapter {
    channel_to_bsc_sender: Sender<Events>,
    channel_to_realis_sender: Sender<Events>
}

impl BSCAdapter {
    pub fn new(channel_to_bsc_sender: Sender<Events>, channel_to_realis_sender: Sender<Events>) -> Self {
        BSCAdapter {
            channel_to_bsc_sender,
            channel_to_realis_sender
        }
    }

    pub async fn listen(&self) {
        let token_listener = BSCListener::new(
            contract::token_new().await,
            self.channel_to_realis_sender.clone()
        );
        let nft_listener = BSCListener::new(
            contract::nft_new().await,
            self.channel_to_realis_sender.clone()
        );
        let token_listener_success = BSCListener::new(
            contract::token_new().await,
            self.channel_to_realis_sender.clone()
        );
        let nft_listener_success = BSCListener::new(
            contract::nft_new().await,
            self.channel_to_realis_sender.clone()
        );

        join!(
            token_listener.listen_token(),
            nft_listener.listen_nft(),
            token_listener_success.listen_token_success(),
            nft_listener_success.listen_nft_success()
        );
    }
}