use primitives::{
    block::Extrinsic,
    events::{RealisEventType, TransferNftToBsc, TransferTokenToBsc},
};
use std::str::FromStr;

use log::{error, info};
use primitives::{events::BridgeExtrinsics, types::BlockNumber};
use runtime::AccountId;
use rust_lib::primitives::adapter::request::token_id_from_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::types::H160;

#[derive(Clone)]
pub struct BlockParser {
    extrinsic: Extrinsic,
    block_number: BlockNumber,
}

impl BlockParser {
    pub fn new(extrinsic: Extrinsic, block_number: BlockNumber) -> Option<Self> {
        if extrinsic.method.pallet == "realisBridge" && extrinsic.method.method == "transferTokenToBsc"
            || extrinsic.method.pallet == "realisBridge" && extrinsic.method.method == "transferNftToBsc"
        {
            info!("Start proccess extrinsic!");
            Some(Self {
                extrinsic,
                block_number,
            })
        } else {
            None
        }
    }

    pub fn parse(self) -> Vec<RealisEventType> {
        error!("Start parse extrinsics {:?}!", self.extrinsic.clone());
        let args = self
            .clone()
            .parse_args(serde_json::from_value::<Args>(self.extrinsic.args.clone()).unwrap());
        match args {
            BridgeExtrinsics::TransferToken(args) => {
                return vec![RealisEventType::TransferTokenToBscSuccess(
                    args,
                    self.extrinsic.hash,
                    self.block_number,
                )]
            }
            BridgeExtrinsics::TransferNft(args) => {
                return vec![RealisEventType::TransferNftToBscSuccess(
                    args,
                    self.extrinsic.hash,
                    self.block_number,
                )]
            }
        }
    }

    pub fn parse_args(self, args: Args) -> BridgeExtrinsics {
        if self.extrinsic.method.method == "transferTokenToBsc" {
            BridgeExtrinsics::TransferToken(TransferTokenToBsc {
                block: self.block_number,
                hash: self.extrinsic.hash,
                from: args.from,
                to: args.to,
                amount: u128::from_str(&serde_json::from_value::<String>(args.value).unwrap()).unwrap(),
            })
        } else {
            BridgeExtrinsics::TransferNft(TransferNftToBsc {
                block: self.block_number,
                hash: self.extrinsic.hash,
                from: args.from,
                dest: args.to,
                token_id: token_id_from_string(args.value).unwrap(),
            })
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Args {
    from: AccountId,
    to: H160,
    value: Value,
}
