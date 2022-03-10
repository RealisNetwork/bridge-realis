pub mod block;
pub mod db;
pub mod types;
pub mod events;

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
    #[error("Error while trying use db_pool: {0}")]
    DbPool(deadpool::managed::PoolError<tokio_postgres::Error>),
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