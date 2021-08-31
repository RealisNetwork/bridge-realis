mod config;
mod request;

pub use config::*;
pub use request::*;

use serde::{Deserialize, Serialize};

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

    Error(String),
}