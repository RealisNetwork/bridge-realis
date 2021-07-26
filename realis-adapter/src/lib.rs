#![cfg_attr(not(feature = "std"), no_std)]

use async_trait::async_trait;
use codec::Decode;
use realis_bridge::TokenId;
use runtime::realis_bridge;
use runtime::Event;
use sp_core::sr25519;
use sp_core::{H160, H256 as Hash};
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use sp_std::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use substrate_api_client::utils::FromHexString;
use substrate_api_client::Api;
use system;
use web3::types::U256;

use logger::logger::{log, Type};

pub struct RealisAdapter<T: BridgeEvents> {
    events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T,
}

impl<T: BridgeEvents> RealisAdapter<T> {
    pub fn new(url: String, event_handler: T) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();
        // Create channels
        let (events_in, events_out) = channel();
        // Subscribe on events
        api.subscribe_events(events_in.clone()).unwrap();

        RealisAdapter {
            events_in,
            events_out,
            event_handler,
        }
    }

    fn parse_events_str(event_str: String) -> Vec<system::EventRecord<Event, Hash>> {
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        return Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc).unwrap();
    }

    async fn process_event(&self, event: &system::EventRecord<Event, Hash>) {
        match &event.event {
            Event::RealisBridge(bridge_event) => match bridge_event {
                realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                    self.event_handler
                        .on_transfer_token_to_bsc(&to, value)
                        .await;
                    log(Type::Info, String::from("From"), &from);
                    log(Type::Info, String::from("To"), &to);
                    log(Type::Info, String::from("Value"), &value);
                }
                realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
                    self.event_handler
                        .on_transfer_nft_to_bsc(&to, &token_id)
                        .await;
                    log(Type::Info, String::from("From"), &from);
                    log(Type::Info, String::from("To"), &to);
                    log(Type::Info, String::from("Token id"), &token_id);
                }
                realis_bridge::Event::TransferTokenToRealis(from, to, value) => {
                    log(Type::Info, String::from("From"), &from);
                    log(Type::Info, String::from("To"), &to);
                    log(Type::Info, String::from("Value"), &value);
                }
                realis_bridge::Event::TransferNftToRealis(from, to, token_id) => {
                    log(Type::Info, String::from("From"), &from);
                    log(Type::Info, String::from("To"), &to);
                    log(Type::Info, String::from("Token id"), &token_id);
                }
                _ => log(
                    Type::Warning,
                    String::from("Unsupported event"),
                    &event.event,
                ),
            },
            _ => log(
                Type::Warning,
                String::from("Unsupported event"),
                &event.event,
            ),
        }
    }

    // Add bsc sender as argument
    pub async fn listener(&self) {
        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events = RealisAdapter::<T>::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event).await;
                    }
                }
                Err(error) => println!("{}", error),
            }
        }
    }
}

#[async_trait]
pub trait BridgeEvents {
    async fn on_transfer_token_to_bsc<'a>(&self, to: &H160, value: &u128);
    async fn on_transfer_nft_to_bsc<'a>(&self, to: &H160, token_id: &TokenId);
}