use crate::events::{
    realis::{TransferNftToBsc, TransferTokenToBsc},
    traits::Event,
};
use ethabi::Token;

use realis_bridge::Call as RealisBridgeCall;
use realis_primitives::TokenId;
use runtime::{AccountId, Call};
use serde::{Deserialize, Serialize};
use substrate_api_client::sp_runtime::app_crypto::sp_core;
use web3::{
    contract::tokens::Tokenize,
    types::{H160, H256, U128, U64},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTokenToRealis {
    pub block: Option<U64>,
    pub hash: H256,
    pub from: H160,
    pub to: AccountId,
    pub amount: u128,
}

impl Event for TransferTokenToRealis {
    fn get_hash(&self) -> String {
        format!("{:?}", self.hash)
    }

    fn get_realis_call(&self) -> Call {
        Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
            sp_core::H160::from_slice(self.from.as_ref()),
            self.to.clone(),
            self.amount * 1_000_000_000_000,
        ))
    }

    // Rollback
    fn get_binance_call(&self) -> (String, Vec<Token>) {
        (
            String::from("transfer"),
            (self.from, U128::from(self.amount)).into_tokens(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNftToRealis {
    pub block: Option<U64>,
    pub hash: H256,
    pub from: H160,
    pub dest: AccountId,
    pub token_id: TokenId,
}

impl Event for TransferNftToRealis {
    fn get_hash(&self) -> String {
        format!("{:?}", self.hash)
    }

    fn get_realis_call(&self) -> Call {
        Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
            sp_core::H160::from_slice(self.from.as_ref()),
            self.dest.clone(),
            self.token_id,
        ))
    }

    fn get_binance_call(&self) -> (String, Vec<Token>) {
        (
            String::from("safeMint"),
            (
                self.dest.to_string(),
                self.from,
                U128::from_dec_str(&self.token_id.to_string()).unwrap(),
            )
                .into_tokens(),
        )
    }
}

#[derive(Debug, Clone)]
pub enum BscEventType {
    TransferTokenToRealis(TransferTokenToRealis),
    TransferNftToRealis(TransferNftToRealis),

    TransferTokenToBscFail(TransferTokenToBsc),
    TransferNftToBscFail(TransferNftToBsc),
}
