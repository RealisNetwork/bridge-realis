use ethabi::Token;
use crate::types::{BlockNumber, Hash};
use crate::events::traits::Event;

use realis_primitives::TokenId;
use runtime::{AccountId, Call, realis_game_api as RealisGameApi};
use serde::{Deserialize, Serialize};
use web3::contract::tokens::Tokenizable;
use web3::types::{H160, U128};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTokenToBsc {
    pub block: BlockNumber,
    pub hash: Hash,
    pub from: AccountId,
    pub to: H160,
    pub amount: u128,
}

impl Event for TransferTokenToBsc {
    // Rollback
    fn get_realis_call(&self) -> Call {
        // TODO not sure about this call
        Call::RealisGameApi(RealisGameApi::Call::transfer_from_pallet(
            self.from.clone(),
            self.amount
        ))
    }

    fn get_binance_call(&self) -> (String, (Token, Token, Token)) {
        (
            String::from("transferFromRealis"),
            (self.from.to_string().into_token(), self.to.into_token(), U128::from(self.amount).into_token())
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNftToBsc {
    pub block: BlockNumber,
    pub hash: Hash,
    pub from: AccountId,
    pub dest: H160,
    pub token_id: TokenId,
}

impl Event for TransferNftToBsc {
    // Rollback
    fn get_realis_call(&self) -> Call {
        todo!()
    }

    fn get_binance_call(&self) -> (String, (Token, Token, Token)) {
        (
            String::from("safeMint"),
            (
                self.from.to_string().into_token(),
                self.dest.into_token(),
                U128::from_dec_str(&self.token_id.to_string()).unwrap().into_token()
            )
        )
    }
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