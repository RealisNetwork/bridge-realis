use std::str::FromStr;
use ethabi::Error;
use realis_primitives::TokenId;
use web3::ethabi::ParamType;
use web3::types::{H256, TransactionReceipt};
use primitives::events::bsc::{BscEventType, TransferNftToRealis, TransferTokenToRealis};
use runtime::AccountId;
use substrate_api_client::sp_runtime::app_crypto::Ss58Codec;

#[derive(Debug)]
pub enum ParseError {
    MissingParam(usize),
    DecodeError(Error),
    AccountId,
    U128,
    Address,
    TokenID,
}

pub trait EventParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>>;
}

pub struct TokenParser {}

impl EventParser for TokenParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>> {
        receipt.logs.iter().filter(|log| log.topics.contains(topic)).map(|log| {
            let params = ethabi::decode(
                &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                &log.data.0,
            ).map_err(ParseError::DecodeError)?;

            let to = AccountId::from_string(&params
                .get(0)
                .ok_or(ParseError::MissingParam(0))?
                .to_string())
                .map_err(|_| ParseError::AccountId)?;

            let amount = params
                .get(1)
                .ok_or(ParseError::MissingParam(1))?
                .clone()
                .into_uint()
                .ok_or(ParseError::U128)?
                .as_u128();

            let from = params
                .get(2)
                .ok_or(ParseError::MissingParam(2))?
                .clone()
                .into_address()
                .ok_or(ParseError::Address)?;

            Ok(BscEventType::TransferTokenToRealis(
                TransferTokenToRealis {
                    block: receipt.block_number,
                    hash: receipt.transaction_hash,
                    from,
                    to,
                    amount
                }
            ))
        }).collect()
    }
}

pub struct NftParser {}

impl EventParser for NftParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>> {
        receipt.logs.iter().filter(|log| log.topics.contains(topic)).map(|log| {
            let params = ethabi::decode(
                &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                &log.data.0,
            ).map_err(ParseError::DecodeError)?;

            let from = params
                .get(0)
                .ok_or(ParseError::MissingParam(0))?
                .clone()
                .into_address()
                .ok_or(ParseError::Address)?;

            let to = AccountId::from_string(&params
                .get(1)
                .ok_or(ParseError::MissingParam(1))?
                .to_string())
                .map_err(|_| ParseError::AccountId)?;

            let token_id = TokenId::from_str(&params
                .get(2)
                .ok_or(ParseError::MissingParam(2))?
                .to_string()
            ).map_err(|_| ParseError::TokenID)?;

            Ok(BscEventType::TransferNftToRealis(
                TransferNftToRealis {
                    block: receipt.block_number,
                    hash: receipt.transaction_hash,
                    from,
                    dest: to,
                    token_id
                }
            ))
        }).collect()
    }
}