#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::{Api, UncheckedExtrinsicV4, compose_extrinsic, XtStatus};
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
use sp_core::sp_std::str::FromStr;
use substrate_api_client::sp_runtime::AccountId32;
use std::convert::TryFrom;

fn main() {
    env_logger::init();
    let url = String::from("rpc.realis.network");

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let (pair, _seed) =
    // TODO not push with phrase
        Pair::from_phrase(
            "",
            Some("")
        ).unwrap();

    let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).map(|api| api.set_signer(pair)).unwrap();

    // set the recipient
    let to: AccountId32 = AccountId32::from_str("").unwrap();

    // call Balances::transfer
    // the names are given as strings
    #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Balances",
        "transfer",
        GenericAddress::Id(to),
        Compact(10_000 as u128)
    );

    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    // send and watch extrinsic until InBlock
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got included. Hash: {:?}", tx_hash);
}