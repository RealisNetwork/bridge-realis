use crate::config::types::{BlockNumber, Hash};
use runtime::AccountId;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Block {
    #[serde(deserialize_with = "u32_from_string")]
    pub number: BlockNumber,
    pub hash: Hash,
    pub parentHash: String,
    pub extrinsicsRoot: String,
    pub authorId: AccountId,
    // logs,
    // onInitialize,
    pub extrinsics: Vec<Extrinsic>,
    // onFinalize,
    pub finalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Extrinsic {
    pub method: Method,
    #[serde(default)]
    signature: Option<Signature>,
    // nonce: Option<String>,
    pub args: Value,
    #[serde(default)]
    pub tip: Option<String>,
    pub hash: Hash,
    // info: ,
    pub events: Vec<Event>,
    pub success: bool,
    pub paysFee: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub method: Method,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub pallet: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signature: String,
    pub signer: Signer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    pub id: AccountId,
}

/// # Errors
pub fn u32_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.parse::<u64>() {
        Ok(value) => Ok(value),
        Err(error) => Err(serde::de::Error::custom(format!(
            "Cannot convert to u64 with error: {:?}",
            error
        ))),
    }
}
