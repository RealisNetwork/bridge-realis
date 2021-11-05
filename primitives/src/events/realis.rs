use crate::types::{BlockNumber, Hash};
use realis_primitives::TokenId;
use runtime::AccountId;
use serde::{Deserialize, Serialize};
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

// TODO can remove Hash and BlockNumber variants from args?
#[derive(Debug, Clone)]
pub enum RealisEventType {
    TransferTokenToBsc(TransferTokenToBsc, Hash, BlockNumber),
    TransferNftToBsc(TransferNftToBsc, Hash, BlockNumber),
}

impl RealisEventType {
    #[must_use]
    pub fn get_hash(&self) -> Hash {
        match self {
            RealisEventType::TransferTokenToBsc(request, ..) => request.hash,
            RealisEventType::TransferNftToBsc(request, ..) => request.hash,
        }
    }
}