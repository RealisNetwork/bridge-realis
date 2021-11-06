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
