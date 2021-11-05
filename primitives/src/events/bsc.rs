use realis_primitives::TokenId;
use runtime::AccountId;
use serde::{Deserialize, Serialize};
use web3::types::{H160, H256, U64};



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

// TODO can remove Hash and Option<U64> variants from args?
#[derive(Debug, Clone)]
pub enum BscEventType {
    TransferTokenToRealis(TransferTokenToRealis, H256, Option<U64>),
    TransferNftToRealis(TransferNftToRealis, H256, Option<U64>),
}



impl BscEventType {
    #[must_use]
    pub fn get_hash(&self) -> H256 {
        match self {
            BscEventType::TransferTokenToRealis(request, _, _) => request.hash,
            BscEventType::TransferNftToRealis(request, _, _) => request.hash,
        }
    }
}
