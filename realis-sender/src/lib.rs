#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::{Api, UncheckedExtrinsicV4, compose_extrinsic, XtStatus, ApiClientError};
use substrate_api_client::utils::FromHexString;
use sp_core::{sr25519, H256};
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
use std::path::Path;
use std::fs;
use sp_core::crypto::SecretStringError;
use logger::logger::{log, Type};
use bsc_adapter::ContractEvents;

// fn main() {
//     env_logger::init();
//     let url = String::from("rpc.realis.network");
//
//     // initialize api and set the signer (sender) that is used to sign the extrinsics
//     let pair = Pair::from_string(&*from_path_to_account("res/accounts.key"), None).unwrap();
//
//     let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).map(|api| api.set_signer(pair)).unwrap();
//
//     // set the recipient
//     let to: AccountId32 = AccountId32::from_str("1aa0d5c594a4581ec17069ec9631cd6225d5fb403fe4d85c8ec8aa51833fdf7f").unwrap();
//
//     // call Balances::transfer
//     // the names are given as strings
//     #[allow(clippy::redundant_clone)]
//         let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
//         api.clone(),
//         "Balances",
//         "transfer",
//         GenericAddress::Id(to),
//         Compact(10_000 as u128)
//     );
//
//     println!("[+] Composed Extrinsic:\n {:?}\n", xt);
//
//     // send and watch extrinsic until InBlock
//     let tx_hash = api
//         .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
//         .unwrap();
//     println!("[+] Transaction got included. Hash: {:?}", tx_hash);
// }

fn from_path_to_account<P: AsRef<Path>>(path: P) -> String {
    let string = fs::read_to_string(path).unwrap();
    return string
}

pub struct RealisSender {
    api: Api<sr25519::Pair>
}

impl RealisSender {
    pub fn new(url: &str) -> Self {
        // Get private key
        let pair =
            Pair::from_string(&*from_path_to_account("./../realis-sender/res/accounts.key"), None).unwrap();
        // Create substrate api with signer
        let api =
            Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
                .map(|api| api.set_signer(pair))
                .unwrap();

        RealisSender {
            api
        }
    }
}

#[async_trait]
impl ContractEvents for RealisSender {
    async fn on_transfer_token_to_realis<'a>(&self, to: AccountId32, value: &u128) {

        // let from: AccountId32 =
        //     AccountId32::from_str("1aa0d5c594a4581ec17069ec9631cd6225d5fb403fe4d85c8ec8aa51833fdf7f")
        //         .unwrap();
        // Create extrinsic transaction
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            self.api.clone(),
            "Balances",
            "transfer",
            GenericAddress::Id(to),
            Compact(10_000 as u128)
        );
        // Send extrinsic transaction
        let tx_result = self.api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => log(Type::Success, String::from("Send extrinsic"), &hash),
            Err(error) => log(Type::Error, String::from("Can`t send extrinsic"), &error)
        }
    }
}