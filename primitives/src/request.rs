use realis_primitives::{Basic, TokenId};
use serde::{Deserialize, Deserializer, Serialize};

pub type Version = String;
pub type Topic = String;
pub type TopicRes = String;
pub type Lang = String;
pub type Id = String;

pub type Amount = u128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Raw<T> {
    pub version: Version,
    pub topic: Topic,
    pub topic_res: TopicRes,
    pub lang: Lang,
    pub id: Id,

    pub params: T,
}

// CreditHardCurrency, DebitHardCurrency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferToBSC {
    pub account_id: String,
    pub bsc_account: String,
    #[serde(deserialize_with = "u128_from_any")]
    pub amount: Amount,
}

// CreditHardCurrency, DebitHardCurrency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferToRealis {
    pub account_id: String,
    pub bsc_account: String,
    #[serde(deserialize_with = "u128_from_any")]
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddNftToBsc {
    pub account_id: String,
    pub bsc_account: String,
    pub token_id: TokenId,
    pub token_type: Basic,
    pub rarity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddNftToRealis {
    pub bsc_account: String,
    pub account_id: String,
    pub token_id: TokenId,
    pub token_type: Basic,
    pub rarity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WithdrawToBsc {
    pub account_id: String,
    pub bsc_account: String,
    #[serde(deserialize_with = "u128_from_any")]
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WithdrawToRealis {
    pub bsc_account: String,
    pub account_id: String,
    #[serde(deserialize_with = "u128_from_any")]
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    Internal(Internal),
    External(External)
}

// Contains error, in adapter executing logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Internal {
    /// Error while connect
    /// Parametr: one of possible connections
    Connection(Connections),
    /// Error while send message to nats
    NatsSend,
    /// Send error, appears when can't send value by channel
    /// Contains two parametrs (from, to)
    ChannelSend(String, String)
}

/// Contains list of all possible connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Connections {
    /// Subscride on nats
    NatsListen,
    /// Send to nats
    NatsSend,
    /// Subscride on blockchain head
    RealisBlockchainListen,
    /// Send to blockchain
    RealisBlockchainSend,
    /// Block getter
    Sidecar,
    /// Database
    Database
}

// Contains error, caused by:
// wrong request, wrong request parametrs...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum External {
    /// Cann't parse request
    ParseRequest,
    /// Blockchain fail while execute
    Extrinsic,
    /// Blockchain storage don't have value
    Storage,
    /// Wallet for user don't exist in DB
    UserWithoutWallet
}

// impl From<Error> for u32 {
//     fn from(error: Error) -> u32 {
//         match error {
//             Internal(_) => 11,
//             External(error) => match error {
//                 ParseRequest => 21,
//                 Extrinsic => 22,
//                 Storage => 23,
//                 UserWithoutWallet => 24
//             }
//         }
//     }
// }

/// # Errors
pub fn u128_from_any<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrU64<'a> {
        Str(&'a str),
        U64(u64),
    }

    Ok(match StrOrU64::deserialize(deserializer)? {
        StrOrU64::Str(v) => v.parse().unwrap_or(0), // Ignoring parsing errors
        StrOrU64::U64(v) => v.into(),
    })
}
