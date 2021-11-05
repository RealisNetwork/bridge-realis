pub mod block;
pub mod db;
pub mod events;
pub mod types;

use substrate_api_client::ApiClientError;
use thiserror::Error;
use web3::Error as Web3Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while send throw channel!")]
    Send,
    #[error("Error while trying use tokio_postgres: {0}")]
    Postgres(tokio_postgres::Error),
    #[error("Error while trying parse: {0}")]
    SerdeJSON(serde_json::error::Error),
    #[error("Disconnected from Database!")]
    Disconnected,
    #[error("Cannot found this file in this path {0}!")]
    FileNotFound(String),
    #[error("Cannot decode this value!")]
    CannotDecode,
    #[error("Realis error: {0}")]
    Api(ApiClientError),
    #[error("Binance error: {0}")]
    Web3(Web3Error),
    #[error("{0}")]
    Custom(String),
}

pub enum Status {
    Got = 0,
    RealisError = 1,
    RealisSuccess = 2,
    BinanceError = 3,
    BinanceSuccess = 4,
    RollbackError = 5,
    RollbackSuccess = 6,
}

impl Status {
    #[must_use]
    pub fn new(number: u32) -> Option<Self> {
        match number {
            0 => Some(Self::Got),
            1 => Some(Self::RealisError),
            2 => Some(Self::RealisSuccess),
            3 => Some(Self::BinanceError),
            4 => Some(Self::BinanceSuccess),
            5 => Some(Self::RollbackError),
            6 => Some(Self::RollbackSuccess),
            _ => None,
        }
    }
}
