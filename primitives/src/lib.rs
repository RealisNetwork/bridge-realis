pub mod block;
pub mod types;
pub mod events;

use thiserror::Error;

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
    #[error("User not found!")]
    NotInteresting,
}