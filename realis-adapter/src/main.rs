#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::BlockNumber;
use substrate_api_client::Api;
use substrate_api_client::utils::FromHexString;
use sp_core::sr25519;
use sp_runtime::generic;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::{channel, Receiver};
use codec::Decode;
use sp_core::H256 as Hash;
use log::{debug, error};
use sp_std::prelude::*;
use system;
use runtime::Event;
use runtime::realis_bridge;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

pub type SignedBlock = generic::SignedBlock<Block>;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

// #[derive(Decode)]
// struct TransferEventArgs {
//     from: AccountId,
//     to: AccountId,
//     value: u128,
// }

fn main() {
    listener_realis()
}

fn listener_realis() {
    // if no signer is set in the whole program, we need to give to Api a specific type instead of an associated type
    // as during compilation the type needs to be defined
    env_logger::init();

    let api = get_api();

    println!("Subscribe to events");
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();

    listener(events_out);
}

fn get_api() -> Api<sr25519::Pair> {
    let url = "rpc.realis.network";
    Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap()
}

fn listener(events_out: Receiver<String>) {
    loop {
        let event = events_out.recv();
        match event {
            Ok(event_str) => {
                let unhex = Vec::from_hex(event_str).unwrap();
                let mut er_enc = unhex.as_slice();
                let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc);
                match _events {
                    Ok(evts) => {
                        for evr in &evts {
                            println!("decoded: {:?} {:?}", evr.phase, evr.event);
                            match &evr.event {
                                Event::RealisBridge(bridge_event) => {
                                    println!("\n\x1b[32mBridge event:\x1b[0m {:?}", bridge_event);
                                    match bridge_event {
                                        realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                                            //sent_to_bsc(to, value);
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
                                    println!()
                                }
                                _ => debug!("ignoring unsupported module event: {:?}", evr.event),
                            }
                        }
                    }
                    Err(_) => error!("couldn't decode event record list"),
                };
            }
            Err(error) => println!("{}", error)
        }
    }
}