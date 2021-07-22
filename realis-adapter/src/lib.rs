#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::BlockNumber;
use substrate_api_client::Api;
use substrate_api_client::utils::FromHexString;
use sp_core::sr25519;
use sp_runtime::generic;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::{channel, Receiver, Sender};
use codec::Decode;
use sp_core::H256 as Hash;
use log::{debug, error};
use sp_std::prelude::*;
use system;
use runtime::Event;
use runtime::realis_bridge;

// fn listener_realis() {
//     // if no signer is set in the whole program, we need to give to Api a specific type instead of an associated type
//     // as during compilation the type needs to be defined
//     env_logger::init();
//
//     let api = get_api();
//
//     println!("Subscribe to events");
//     let (events_in, events_out) = channel();
//     api.subscribe_events(events_in).unwrap();
//
//     listener(events_out);
// }
//
// // fn get_api() -> Api<sr25519::Pair> {
// //     let url = "rpc.realis.network";
// //     Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap()
// // }
//
// fn listener(events_out: Receiver<String>) {
//     loop {
//         let event = events_out.recv();
//         match event {
//             Ok(event_str) => {
//                 let unhex = Vec::from_hex(event_str).unwrap();
//                 let mut er_enc = unhex.as_slice();
//                 let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc);
//                 match _events {
//                     Ok(evts) => {
//                         for evr in &evts {
//                             println!("decoded: {:?} {:?}", evr.phase, evr.event);
//                             match &evr.event {
//                                 Event::RealisBridge(bridge_event) => {
//                                     println!("\n\x1b[32mBridge event:\x1b[0m {:?}", bridge_event);
//                                     match bridge_event {
//                                         realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
//                                             //sent_to_bsc(to, value);
//                                             println!("From: {:?}", from);
//                                             println!("To: {:?}", to);
//                                             println!("Value: {:?}", value);
//                                         }
//                                         realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
//                                             println!("From: {:?}", from);
//                                             println!("To: {:?}", to);
//                                             println!("Value: {:?}", token_id);
//                                         }
//                                         realis_bridge::Event::TransferTokenToRealis(from, to, value) => {
//                                             println!("From: {:?}", from);
//                                             println!("To: {:?}", to);
//                                             println!("Value: {:?}", value);
//                                         }
//                                         realis_bridge::Event::TransferNftToRealis(from, to, token_id) => {
//                                             println!("From: {:?}", from);
//                                             println!("To: {:?}", to);
//                                             println!("Value: {:?}", token_id);
//                                         }
//                                         _ => println!("\x1b[31mUnsupported event!\x1b[0m")
//                                     }
//                                     println!()
//                                 }
//                                 // TODO add Nft and realisGameApi mint/burn event
//                                 _ => debug!("ignoring unsupported module event: {:?}", evr.event),
//                             }
//                         }
//                     }
//                     Err(_) => error!("couldn't decode event record list"),
//                 };
//             }
//             Err(error) => println!("{}", error)
//         }
//     }
// }

struct RealisAdapter {
    events_in: Sender<String>,
    events_out: Receiver<String>
}

impl RealisAdapter {

    pub fn new(url: String) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();
        // Create channels
        let (events_in, events_out) = channel();
        // Subscribe on events
        api.subscribe_events(events_in.clone()).unwrap();

        RealisAdapter {
            events_in,
            events_out
        }
    }

    fn parse_events_str(event_str: String) -> Vec::<system::EventRecord<Event, Hash>> {
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        return Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc).unwrap();
    }

    // Add bsc sender as argument
    pub fn listen(&self) {
        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events = RealisAdapter::parse_events_str(event_str);
                    for event in &events {
                        match &event.event {
                            Event::RealisBridge(bridge_event) => {
                                match bridge_event {
                                    realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                                        println!("From: {:?}", from);
                                        println!("To: {:?}", to);
                                        println!("Value: {:?}", value);
                                    }
                                    realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
                                        println!("From: {:?}", from);
                                        println!("To: {:?}", to);
                                        println!("Value: {:?}", token_id);
                                    }
                                    realis_bridge::Event::TransferTokenToRealis(from, to, value) => {
                                        println!("From: {:?}", from);
                                        println!("To: {:?}", to);
                                        println!("Value: {:?}", value);
                                    }
                                    realis_bridge::Event::TransferNftToRealis(from, to, token_id) => {
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
                }
                Err(error) => println!("{}", error)
            }
        }
    }
}

trait BridgeEvents {
    fn on_transfer_token_to_bsc();
    fn on_transfer_nft_to_bsc();
    fn
}