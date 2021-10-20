use crate::types::{BlockNumber, Hash};
use realis_primitives::TokenId;
use runtime::AccountId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use web3::types::H160;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTokenToBsc {
    pub block: BlockNumber,
    pub hash: Hash,
    pub from: AccountId,
    pub to: H160,
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNftToBsc {
    pub block: BlockNumber,
    pub hash: Hash,
    pub from: AccountId,
    pub dest: H160,
    pub token_id: TokenId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeExtrinsics {
    TransferNft(TransferNftToBsc),
    TransferToken(TransferTokenToBsc),
}

#[derive(Debug, Clone)]
pub enum EventType {
    TransferTokenToBscSuccess(TransferTokenToBsc, Hash, BlockNumber),
    TransferTokenToBscError(TransferTokenToBsc, Hash, BlockNumber),
    TransferNftToBscSuccess(TransferNftToBsc, Hash, BlockNumber),
    TransferNftToBscError(TransferNftToBsc, Hash, BlockNumber),
}
