#![cfg_attr(not(feature = "std"), no_std)]

use sp_keyring::AccountKeyring;
use substrate_api_client::{AccountInfo, BlockNumber};
use substrate_api_client::Api;
use substrate_api_client::utils::FromHexString;
use sp_core::sr25519;
use sp_runtime::generic;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::channel;
use codec::Decode;
use sp_core::H256 as Hash;
use log::{debug, error};
use sp_std::prelude::*;
use substrate_api_client::extrinsic::xt_primitives::AccountId;
use system;
use balances;
use runtime::Event;

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
                        Event::Balances(be) => {
                            println!(">>>>>>>>>> balances event: {:?}", be);
                            match &be {
                                balances::Event::Transfer(transactor, dest, value) => {
                                    println!("Transactor: {:?}", transactor);
                                    println!("Destination: {:?}", dest);
                                    println!("Value: {:?}", value);
                                }
                                _ => {
                                    debug!("ignoring unsupported balances event");
                                }
                            }
                        }
                        _ => debug!("ignoring unsupported module event: {:?}", evr.event),
                    }
                }
            }
            Err(_) => error!("couldn't decode event record list"),
        }
    }
}

fn get_api() -> Api<sr25519::Pair> {
    let url = "rpc.realis.network";
    Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap()
}