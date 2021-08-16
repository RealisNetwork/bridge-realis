use log::info;
use primitive_types::U256;
use ratsio::StanMessage;
use realis_primitives::{Basic, Rarity, TokenId};
use runtime::AccountId;
use serde::{Deserialize, Serialize};
use serde_json::{self, from_str, Map, Value};
use sp_core::{crypto::Ss58Codec, H160};
use std::{convert::From, str::FromStr};
use thiserror::Error;

/// # Errors
pub fn parse(message: &StanMessage) -> Result<Request, Error> {
    convert_message(message)
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

pub type UserId = String;
pub type TransactionHash = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRequest {
    pub method: String,
    pub params: Map<String, serde_json::Value>,
    pub id: String,
    pub agent: String,
    pub lang: String,
}

#[derive(Debug, Clone)]
pub enum Request {
    TransferFromRealis {
        user_id: UserId,
        account_id: AccountId,
        bsc_account: H160,
        amount: U256,
        id: String,
        agent: String,
        lang: String,
    },
    TransferFromRealisNft {
        user_id: UserId,
        account_id: AccountId,
        bsc_account: H160,
        token_id: U256,
        token_type: Basic,
        rarity: Rarity,
        id: String,
        agent: String,
        lang: String,
    },
    SendToRealis {
        user_id: UserId,
        bsc_account: H160,
        account_id: AccountId,
        amount: U256,
        id: String,
        agent: String,
        lang: String,
    },
    SendToRealisNft {
        user_id: UserId,
        account_id: AccountId,
        bsc_account: H160,
        token_id: TokenId,
        token_type: Basic,
        rarity: Rarity,
        id: String,
        agent: String,
        lang: String,
    },
}

/// # Errors
pub fn convert_message(message: &StanMessage) -> Result<Request, Error> {
    // Convert to json value object
    let raw_request: Result<RawRequest, serde_json::Error> =
        serde_json::from_slice(&message.payload);
    match raw_request {
        Ok(raw_request) => {
            // Parse json from string to objects
            info!("Get JSON: {:?}", raw_request);
            // Parse RawRequest to Request
            parse_json(raw_request)
        }
        Err(_) => Err(Error::Parse),
    }
}

// TODO: why dont derive serde::Deserialize?
fn parse_json(raw_request: RawRequest) -> Result<Request, Error> {
    // Get params from JSON
    let params = raw_request.params;
    // Parse common fields
    let id = raw_request.id;
    let agent = raw_request.agent;
    let lang = raw_request.lang;
    // Parse request params depending on the calling method
    match raw_request.method.as_str() {
        "transfer_from_realis" => {
            let user_id: UserId = get_string(&params, "user_id")?.to_owned();
            let account_id = parse_account_id(&params, "account_id")?;
            let bsc_account = parse_account_bsc(&params, "bsc_account")?;
            let amount = parse_u256(&params, "amount")?;
            Ok(Request::TransferFromRealis {
                user_id,
                account_id,
                bsc_account,
                amount,
                id,
                agent,
                lang,
            })
        }
        "transfer_from_realis_nft" => {
            let user_id: UserId = get_string(&params, "user_id")?.to_owned();
            let token_id: U256 = parse_u256(&params, "token_id")?;
            let token_type: Basic = parse_basic(&params, "token_type")?;
            let rarity: Rarity = parse_rarity(&params, "rarity")?;
            let account_id = parse_account_id(&params, "account_id")?;
            let bsc_account = parse_account_bsc(&params, "bsc_account")?;
            Ok(Request::TransferFromRealisNft {
                user_id,
                account_id,
                bsc_account,
                token_id,
                token_type,
                rarity,
                id,
                agent,
                lang,
            })
        }
        "transfer_from_bsc" => {
            let user_id: UserId = get_string(&params, "user_id")?.to_owned();
            let amount: U256 = parse_u256(&params, "amount")?;
            let account_id = parse_account_id(&params, "account_id")?;
            let bsc_account = parse_account_bsc(&params, "bsc_account")?;
            Ok(Request::SendToRealis {
                user_id,
                amount,
                id,
                agent,
                lang,
                account_id,
                bsc_account,
            })
        }
        "transfer_from_bsc_from" => {
            let user_id: UserId = get_string(&params, "user_id")?.to_owned();
            let token_id: TokenId = parse_token_id(&params, "token_id")?;
            let token_type: Basic = parse_basic(&params, "token_type")?;
            let rarity: Rarity = parse_rarity(&params, "rarity")?;
            let account_id = parse_account_id(&params, "account_id")?;
            let bsc_account = parse_account_bsc(&params, "bsc_account")?;
            Ok(Request::SendToRealisNft {
                user_id,
                account_id,
                bsc_account,
                token_id,
                token_type,
                rarity,
                id,
                agent,
                lang,
            })
        }
        method_name => Err(Error::UnknownMethod(String::from(method_name))),
    }
}

fn get_string<'a>(
    params: &'a Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<&'a str, Error> {
    // Get value by field name
    match params.get(field_name) {
        // If field missing
        None => Err(Error::MissingField(String::from(field_name))),
        // If field exist
        Some(value) => string_from_value(value),
    }
}

fn parse_u256(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<U256, Error> {
    // Try to get value
    let value = get_string(params, field_name)?;
    // Try convert value to 'u128'
    u256_from_string(value)
}

fn parse_token_id(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<TokenId, Error> {
    let value = get_string(params, field_name)?;
    token_id_from_string(value)
}

fn parse_basic(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<Basic, Error> {
    let value = get_string(params, field_name)?;
    basic_from_string(value)
}

fn parse_rarity(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<Rarity, Error> {
    let value = get_string(params, field_name)?;
    rarity_from_string(value)
}

fn parse_account_id(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<AccountId, Error> {
    let value = get_string(params, field_name)?;
    ss58_from_string(value)
}

fn parse_account_bsc(
    params: &Map<String, serde_json::Value>,
    field_name: &'static str,
) -> Result<H160, Error> {
    let value = get_string(params, field_name)?;
    h160_from_string(value)
}

fn string_from_value(value: &Value) -> Result<&str, Error> {
    match value.as_str() {
        None => Err(Error::Convert(
            format!("{:?}", value),
            String::from("String"),
        )),
        Some(value) => Ok(value),
    }
}

fn u256_from_string(value: &str) -> Result<U256, Error> {
    match value.parse() {
        // If cannot be converted to 'u128'
        Err(_) => Err(Error::Convert(value.to_string(), String::from("U256"))),
        Ok(amount) => Ok(amount),
    }
}

fn h160_from_string(value: &str) -> Result<H160, Error> {
    match H160::from_str(value) {
        // If cannot be converted to 'u128'
        Err(_) => Err(Error::Convert(value.to_string(), String::from("H160"))),
        Ok(bsc_account) => Ok(bsc_account),
    }
}

fn ss58_from_string(value: &str) -> Result<AccountId, Error> {
    match AccountId::from_ss58check(value) {
        // If cannot be converted to 'u128'
        Err(_) => Err(Error::Convert(value.to_string(), String::from("H160"))),
        Ok(account_id) => Ok(account_id),
    }
}

fn token_id_from_string(value: &str) -> Result<TokenId, Error> {
    match TokenId::from_str_radix(value, 10) {
        Err(_) => Err(Error::Convert(value.to_string(), String::from("TokenId"))),
        Ok(token_id) => Ok(token_id),
    }
}

fn basic_from_string(value: &str) -> Result<Basic, Error> {
    match from_str(value) {
        Err(_) => Err(Error::Convert(value.to_string(), String::from("Basic"))),
        Ok(basic) => Ok(basic),
    }
}

fn rarity_from_string(value: &str) -> Result<Rarity, Error> {
    match value.parse() {
        Ok(rarity) => Ok(rarity),
        Err(_) => Err(Error::UnknownRarity(String::from(value))),
    }
}
