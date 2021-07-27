#![cfg_attr(not(feature = "std"), no_std)]

use substrate_api_client::{Api, UncheckedExtrinsicV4, compose_extrinsic, XtStatus};
use sp_core::sr25519;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use sp_std::prelude::*;
use async_trait::async_trait;
use sp_core::Pair;
use substrate_api_client::sp_runtime::AccountId32;
use std::path::Path;
use std::fs;
use logger::logger::{log, Type};
use bsc_adapter::ContractEvents;

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
            Compact(*value * 10_000_000_000)
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