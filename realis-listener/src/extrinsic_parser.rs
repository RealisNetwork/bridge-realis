use primitives::block::Extrinsic;
use std::str::FromStr;

use log::{error, info};
use primitives::{
    events::realis::{BridgeExtrinsics, RealisEventType, TransferNftToBsc, TransferTokenToBsc},
    types::BlockNumber,
};
use runtime::AccountId;
use rust_lib::primitives::adapter::request::token_id_from_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::types::H160;

#[derive(Clone)]
pub struct ExtrinsicParser {
    extrinsic: UncheckedExtrinsic<Address, Call, Signature, Extra>,
    block_number: BlockNumber,
}

// TODO remove file
impl ExtrinsicParser {
    #[allow(clippy::nonminimal_bool)]
    #[must_use]
    pub fn new(extrinsic: UncheckedExtrinsic<Address, Call, Signature, Extra>, block_number: BlockNumber) -> Option<Self> {
        if let Some((address, signature, extra)) = extrinsic.signature.unwrap_or((Default::default(), Default::default(), ())) {
            let z = address;
            let x = extrinsic.signature.unwrap().0;
        } else {}
            // && extrinsic.method.method == "transferTokenToBsc"
            // && extrinsic
            //     .events
            //     .iter()
            //     .any(|x| x.method.method.contains("ExtrinsicSuccess"))
            // || extrinsic.method.pallet == "realisBridge"
            //     && extrinsic.method.method == "transferNftToBsc"
            //     && extrinsic
            //         .events
            //         .iter()
            //         .any(|x| x.method.method.contains("ExtrinsicSuccess"))
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

    #[must_use]
    /// # Panics
    pub fn parse(self) -> Vec<RealisEventType> {
        error!("Start parse extrinsics {:?}!", self.extrinsic);
        let args = self
            .clone()
            .parse_args(serde_json::from_value::<Args>(self.extrinsic.args).unwrap());
        match args {
            BridgeExtrinsics::TransferToken(args) => return vec![RealisEventType::TransferTokenToBsc(args)],
            BridgeExtrinsics::TransferNft(args) => return vec![RealisEventType::TransferNftToBsc(args)],
        }
    }

    #[must_use]
    /// # Panics
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
