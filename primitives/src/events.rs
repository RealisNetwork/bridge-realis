use crate::types::{BlockNumber, Hash};
use realis_primitives::TokenId;
use runtime::AccountId;
use serde::{Deserialize, Serialize};
use web3::types::{H160, H256, U64};

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
pub struct TransferTokenToRealis {
    pub block: Option<U64>,
    pub hash: H256,
    pub from: H160,
    pub to: AccountId,
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNftToRealis {
    pub block: Option<U64>,
    pub hash: H256,
    pub from: H160,
    pub dest: AccountId,
    pub token_id: TokenId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeExtrinsics {
    TransferNft(TransferNftToBsc),
    TransferToken(TransferTokenToBsc),
}

#[derive(Debug, Clone)]
pub enum RealisEventType {
    TransferTokenToBscSuccess(TransferTokenToBsc, Hash, BlockNumber),
    TransferTokenToBscError(TransferTokenToBsc, Hash, BlockNumber),
    TransferNftToBscSuccess(TransferNftToBsc, Hash, BlockNumber),
    TransferNftToBscError(TransferNftToBsc, Hash, BlockNumber),
}

#[derive(Debug, Clone)]
pub enum BscEventType {
    TransferTokenToRealisSuccess(TransferTokenToRealis, H256, Option<U64>),
    TransferNftToRealisSuccess(TransferNftToRealis, H256, Option<U64>),
    TransferTokenToRealisError(TransferTokenToRealis, H256, Option<U64>),
    TransferNftToRealisError(TransferNftToRealis, H256, Option<U64>),
}
