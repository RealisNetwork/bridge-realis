use realis_primitives::{Basic, Rarity, TokenId};
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
