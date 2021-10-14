use crate::types::{BlockNumber, Hash};
use serde::{Deserialize, Serialize};
use realis_primitives::TokenId;
use serde_json::{json, Value};
use runtime::AccountId;
use web3::types::H160;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTokenToBsc {
    pub from: AccountId,
    pub to: H160,
    pub amount: u128
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNftToBsc {
    pub from: AccountId,
    pub to: H160,
    pub token_id: TokenId
}

#[derive(Debug, Clone)]
pub enum EventType {
    TransferTokenToBscSuccess(TransferTokenToBsc, Hash, BlockNumber),
    TransferTokenToBscError(TransferTokenToBsc, Hash, BlockNumber),
    TransferNftToBscSuccess(TransferNftToBsc, Hash, BlockNumber),
    TransferNftToBscError(TransferNftToBsc, Hash, BlockNumber),
}
