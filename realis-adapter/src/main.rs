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
    let url = "localhost:9944";
    Api::<sr25519::Pair>::new(format!("ws://{}", url)).unwrap()
}

fn listener(events_out: Receiver<String>) {
    loop {
        let event_str = events_out.recv().unwrap();
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc);
        match _events {
            Ok(evts) => {
                for evr in &evts {
                    println!("decoded: {:?} {:?}", evr.phase, evr.event);
                    match &evr.event {
                        Event::RealisBridge(be) => {
                            println!(">>>>>>>>>> balances event: {:?}", be);
                            match be {
                                realis_bridge::Event::TransferTokenToBSC(transactor, dest, value) => {
                                    println!("Transactor: {:?}", transactor);
                                    println!("Destination: {:?}", dest);
                                    println!("Value: {:?}", value);
                                }
                                _ => {
                                    debug!("ignoring unsupported balances event");
                                }
                            }
                        }
                        //Event::Bridge(bridge_event) => {
                        //  println!("Bridge event: {:?}", bridge_event);
                        //  match bridge_event {
                        //      TransferToken(from, to, amount) => {}
                        //      TransferNft(from, to, token_id) => {}
                        //  }
                        //}
                        _ => debug!("ignoring unsupported module event: {:?}", evr.event),
                    }
                }
            }
            Err(_) => error!("couldn't decode event record list"),
        };
    }
}