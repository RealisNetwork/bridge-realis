mod config;
mod request;

pub use config::*;
pub use request::*;

use serde::{Deserialize, Serialize};
use std::convert::From;
use thiserror::Error;

pub type UserId = String;
pub type TransactionHash = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method")]
pub enum Request {
    #[serde(rename = "transfer_token_to_bsc")]
    TransferTokenToBSC(Raw<TransferToBSC>),

    #[serde(rename = "transfer_nft_to_bsc")]
    TransferNftToBSC(Raw<AddNftToBsc>),

    #[serde(rename = "transfer_token_to_realis")]
    TransferTokenToRealis(Raw<TransferToRealis>),

    #[serde(rename = "transfer_nft_to_realis")]
    TransferNftToRealis(Raw<AddNftToRealis>),

    #[serde(rename = "withdraw_from_bsc")]
    WithdrawFromBSC(Raw<WithdrawToBsc>),

    #[serde(rename = "withdraw_from_bsc")]
    WithdrawFromRealis(Raw<WithdrawToRealis>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponderRequest {
    TransferTokenToBSC(Raw<TransferToBSC>),

    TransferNftToBSC(Raw<AddNftToBsc>),

    TransferTokenToRealis(Raw<TransferToRealis>),

    TransferNftToRealis(Raw<AddNftToRealis>),

    Error(),
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Cannot find required field: {0}!")]
    MissingField(String),
    #[error("Find unknown method: {0}!")]
    UnknownMethod(String),
    #[error("Cannot convert {0} to `{1}`!")]
    Convert(String, String),
    #[error("Unknown rarity type: {0}")]
    UnknownRarity(String),
    #[error("Cannot found `user_id`: {0} in database!")]
    UserNotFound(UserId),
    #[error("Cannot parse json!")]
    Parse,
}

impl From<Error> for u32 {
    fn from(item: Error) -> Self {
        match item {
            Error::MissingField(_) => 1,
            Error::UnknownMethod(_) => 2,
            Error::Convert(_, _) => 3,
            Error::UnknownRarity(_) => 4,
            Error::UserNotFound(_) => 5,
            Error::Parse => 6,
        }
    }
}
