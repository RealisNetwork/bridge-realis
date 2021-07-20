#![cfg_attr(not(feature = "std"), no_std)]

use sp_keyring::AccountKeyring;
use substrate_api_client::{AccountInfo, BlockNumber};
use substrate_api_client::Api;
use sp_core::sr25519;
use sp_runtime::generic;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::channel;
use hex::FromHex;
use sp_core::sp_std::hash::Hash;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

pub type SignedBlock = generic::SignedBlock<Block>;


pub type Block = generic::Block<Header, UncheckedExtrinsic>;

fn main() {
    // if no signer is set in the whole program, we need to give to Api a specific type instead of an associated type
    // as during compilation the type needs to be defined
    env_logger::init();
    let url = "rpc.realis.network";

    let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();

    println!("Subscribe to events");

    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();

    loop {
        let event_str = events_out.recv().unwrap();
        let _unhex = Vec::from_hex(event_str).unwrap();
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut _er_enc);

        println!("{}", event_str)
    }
}
