#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::{Api, UncheckedExtrinsicV4, compose_extrinsic};
use substrate_api_client::utils::FromHexString;
use sp_core::sr25519;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::sync::mpsc::{channel, Receiver, Sender};
use codec::Decode;
use sp_core::{H256 as Hash, H160};
use sp_std::prelude::*;
use system;
use runtime::Event;
use runtime::AccountId;
use runtime::realis_bridge;
use realis_bridge::TokenId;
use web3::types::U256;
use async_trait::async_trait;
use hex_literal::hex;
use sp_keyring::AccountKeyring;
use sp_core::Pair;

fn main() {
    env_logger::init();
    let url = "rpc.realis.network";

    let signer: Pair::Pair = Pair::from_seed("10f908b91793b30fc4870e255a0e102745e2a8f268814cd28389ba7f4220764d");

    let api = Api::new(format!("wss://{}", url))
        .map(|api| api.set_signer(signer.clone()))
        .unwrap();

    // let xt = substrate_api_client::Metadata::module((), ("realisBridge"));
        // compose_extrinsic!(api, "RealisBridge", "TransferTokenToRealis", 10_u128);

    // println!("[+] Extrinsic: {:?}\n", xt);
}