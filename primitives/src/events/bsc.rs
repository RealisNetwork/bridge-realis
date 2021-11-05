use ethabi::Token;
use crate::events::traits::Event;
use crate::events::realis::{TransferTokenToBsc, TransferNftToBsc};

use realis_primitives::TokenId;
use runtime::{AccountId, Call};
use serde::{Deserialize, Serialize};
use web3::types::{H160, H256, U64};
use substrate_api_client::sp_runtime::app_crypto::sp_core;
use realis_bridge::Call as RealisBridgeCall;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTokenToRealis {
    pub block: Option<U64>,
    pub hash: H256,
    pub from: H160,
    pub to: AccountId,
    pub amount: u128,
}

impl Event for TransferTokenToRealis {
    fn get_realis_call(&self) -> Call {
        Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
            sp_core::H160::from_slice(self.from.as_ref()),
            self.to.clone(),
            self.amount * 1_000_000_000_000
        ))
    }

    // Rollback
    fn get_binance_call(&self) -> (String, (Token, Token, Token)) {
        todo!()
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
    fn get_realis_call(&self) -> Call {
        Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
            sp_core::H160::from_slice(self.from.as_ref()),
            self.dest.clone(),
            self.token_id,
        ))
    }

    fn get_binance_call(&self) -> (String, (Token, Token, Token)) {
        todo!()
    }
}

// TODO can remove Hash and Option<U64> variants from args?
#[derive(Debug, Clone)]
pub enum BscEventType {
    TransferTokenToRealis(TransferTokenToRealis, H256, Option<U64>),
    TransferNftToRealis(TransferNftToRealis, H256, Option<U64>),

    TransferTokenToBscFail(TransferTokenToBsc),
    TransferNftToBscFail(TransferNftToBsc),
}

impl BscEventType {
    pub fn get_call(&self) -> Call {
        match self {
            BscEventType::TransferTokenToRealis(request, ..) => request.get_realis_call(),
            BscEventType::TransferNftToRealis(request, ..) => request.get_realis_call(),
            BscEventType::TransferTokenToBscFail(request) => request.get_realis_call(),
            BscEventType::TransferNftToBscFail(request) => request.get_realis_call(),
        }
    }
}

impl BscEventType {
    #[must_use]
    pub fn get_hash(&self) -> String {
        match self {
            BscEventType::TransferTokenToRealis(request, _, _) => request.hash.to_string(),
            BscEventType::TransferNftToRealis(request, _, _) => request.hash.to_string(),
            BscEventType::TransferTokenToBscFail(request) => request.hash.to_string(),
            BscEventType::TransferNftToBscFail(request) => request.hash.to_string(),
        }
    }
}
