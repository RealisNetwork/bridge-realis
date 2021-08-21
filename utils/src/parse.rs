use realis_primitives::{Basic, Rarity, TokenId};
use serde::{Deserialize, Deserializer, Serialize};

pub type Version = String;
pub type Topic = String;
pub type TopicRes = String;
pub type Lang = String;
pub type Id = String;

pub type Amount = u128;
pub type UserId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Raw<T> {
    pub version: Version,
    pub topic: Topic,
    pub topic_res: TopicRes,
    pub lang: Lang,
    pub id: Id,

    pub params: T,
}

// CreateWalletPArams, GetBalanceParams, GetNFtItemLists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnlyUser {
    pub user_id: UserId,
}

// CreditHardCurrency, DebitHardCurrency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transfer {
    pub user_id: UserId,
    #[serde(deserialize_with = "u128_from_any")]
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddNftItem {
    pub user_id: UserId,
    pub token_id: TokenId,
    pub token_type: Basic,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddNftList {
    pub user_id: UserId,
    pub tokens: Vec<(TokenId, Basic, Rarity)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveNftItem {
    pub user_id: UserId,
    pub token_id: TokenId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveNftList {
    pub user_id: UserId,
    pub tokens: Vec<TokenId>,
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
