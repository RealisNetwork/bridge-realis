#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::Api;
use substrate_api_client::utils::FromHexString;
use sp_core::sr25519;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::{channel, Receiver, Sender};
use codec::Decode;
use sp_core::{H256 as Hash, H160};
use sp_std::prelude::*;
use system;
use runtime::Event;
use runtime::realis_bridge;
use realis_bridge::TokenId;
use web3::types::U256;


pub struct RealisAdapter<T: BridgeEvents> {
    events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T
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
            event_handler
        }
    }

    fn parse_events_str(event_str: String) -> Vec::<system::EventRecord<Event, Hash>> {
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        return Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc).unwrap();
    }

    fn process_event(&self, event: &system::EventRecord<Event, Hash>) {
        match &event.event {
            Event::RealisBridge(bridge_event) => {
                match bridge_event {
                    realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                        self.event_handler.on_transfer_token_to_bsc(&to, value);
                        println!("From: {:?}", from);
                        println!("To: {:?}", to);
                        println!("Value: {:?}", value);
                    }
                    realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
                        self.event_handler.on_transfer_nft_to_bsc(&to, &token_id);
                        println!("From: {:?}", from);
                        println!("To: {:?}", to);
                        println!("Value: {:?}", token_id);
                    }
                    realis_bridge::Event::TransferTokenToRealis(from, to, value) => {
                        self.event_handler.on_transfer_token_to_realis(&to, &value);
                        println!("From: {:?}", from);
                        println!("To: {:?}", to);
                        println!("Value: {:?}", value);
                    }
                    realis_bridge::Event::TransferNftToRealis(from, to, token_id) => {
                        self.event_handler.on_transfer_nft_to_realis(&to, &token_id);
                        println!("From: {:?}", from);
                        println!("To: {:?}", to);
                        println!("Value: {:?}", token_id);
                    }
                    _ => println!("\x1b[31mUnsupported event!\x1b[0m")
                }
            }
            _ => println!("Unsupported module event: {:?}", event.event),
        }
    }

    // Add bsc sender as argument
    pub fn listener(&self) {
        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events = RealisAdapter::<T>::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event);
                    }
                }
                Err(error) => println!("{}", error)
            }
        }
    }
}


pub trait BridgeEvents {
    fn on_transfer_token_to_bsc(&self, to: &H160, value: &u128);
    fn on_transfer_nft_to_bsc(&self, to: &H160, token_id: &TokenId);
    fn on_transfer_token_to_realis(&self, to: &runtime::AccountId, value: &u128);
    fn on_transfer_nft_to_realis(&self, to: &runtime::AccountId, token_id: &U256);
}