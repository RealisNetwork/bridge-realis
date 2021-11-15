use ethabi::Error;
use primitives::{
    events::bsc::{BscEventType, TransferNftToRealis, TransferTokenToRealis},
    types::RawEvent,
};
use realis_primitives::TokenId;
use runtime::AccountId;
use sp_core::crypto::Ss58Codec;
use std::str::FromStr;
use web3::{
    ethabi::ParamType,
    types::{TransactionReceipt, H256},
};

#[derive(Debug)]
pub enum ParseError {
    MissingParam(RawEvent, usize),
    DecodeError(RawEvent, Error),
    AccountId(RawEvent),
    U128(RawEvent),
    Address(RawEvent),
    TokenID(RawEvent),
}

impl ParseError {
    pub fn get_event(&self) -> RawEvent {
        match self {
            ParseError::MissingParam(event, _)
            | ParseError::DecodeError(event, _)
            | ParseError::AccountId(event)
            | ParseError::U128(event)
            | ParseError::Address(event)
            | ParseError::TokenID(event) => event,
        }
        .clone()
    }
}

pub trait EventParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>>;
}

pub struct TokenParser {}

impl EventParser for TokenParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>> {
        receipt
            .logs
            .iter()
            .filter(|log| log.topics.contains(topic))
            .map(|log| {
                let raw_event = RawEvent {
                    block_number: receipt.block_number,
                    hash: receipt.transaction_hash,
                    data: log.data.0.clone(),
                };

                let params = ethabi::decode(
                    &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                    &log.data.0,
                )
                .map_err(|error| ParseError::DecodeError(raw_event.clone(), error))?;

                let to = AccountId::from_string(
                    &params
                        .get(0)
                        .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 0))?
                        .to_string(),
                )
                .map_err(|_| ParseError::AccountId(raw_event.clone()))?;

                let amount = params
                    .get(1)
                    .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 1))?
                    .clone()
                    .into_uint()
                    .ok_or_else(|| ParseError::U128(raw_event.clone()))?
                    .as_u128();

                let from = params
                    .get(2)
                    .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 2))?
                    .clone()
                    .into_address()
                    .ok_or_else(|| ParseError::Address(raw_event.clone()))?;

                Ok(BscEventType::TransferTokenToRealis(TransferTokenToRealis {
                    block: receipt.block_number,
                    hash: receipt.transaction_hash,
                    from,
                    to,
                    amount,
                }))
            })
            .collect()
    }
}

pub struct NftParser {}

impl EventParser for NftParser {
    fn parse(receipt: TransactionReceipt, topic: &H256) -> Vec<Result<BscEventType, ParseError>> {
        receipt
            .logs
            .iter()
            .filter(|log| log.topics.contains(topic))
            .map(|log| {
                let raw_event = RawEvent {
                    block_number: receipt.block_number,
                    hash: receipt.transaction_hash,
                    data: log.data.0.clone(),
                };

                let params = ethabi::decode(
                    &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                    &log.data.0,
                )
                .map_err(|error| ParseError::DecodeError(raw_event.clone(), error))?;

                let from = params
                    .get(0)
                    .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 0))?
                    .clone()
                    .into_address()
                    .ok_or_else(|| ParseError::Address(raw_event.clone()))?;

                let to = AccountId::from_string(
                    &params
                        .get(1)
                        .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 1))?
                        .to_string(),
                )
                .map_err(|_| ParseError::AccountId(raw_event.clone()))?;
                let token_id = TokenId::from_str(
                    &params
                        .get(2)
                        .ok_or_else(|| ParseError::MissingParam(raw_event.clone(), 2))?
                        .to_string(),
                )
                .map_err(|_| ParseError::TokenID(raw_event.clone()))?;

                Ok(BscEventType::TransferNftToRealis(TransferNftToRealis {
                    block: receipt.block_number,
                    hash: receipt.transaction_hash,
                    from,
                    dest: to,
                    token_id,
                }))
            })
            .collect()
    }
}
