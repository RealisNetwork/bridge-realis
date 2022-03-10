use std::time::Duration;
use crate::config::{types::BlockNumber, Error};

use crate::config::{
    db::Status,
    events::{bsc::BscEventType, realis::RealisEventType},
    types::RawEvent,
};
use rust_lib::{
    inner_db::{client_inner::DatabaseClientInner, client_inner_builder::DatabaseClientInnerBuilder},
};
use rust_lib::inner_db::client_inner_builder::BuildError;
use web3::ethabi::ethereum_types::U64;
use crate::config::Error::DbPool;
use backoff::future::retry;
use realis_macros::macro_retry;

pub struct Database {
    client: DatabaseClientInner,
}

impl Database {
    /// # Panics
    /// # Errors
    pub async fn new(
        host: String,
        port: u16,
        user: String,
        password: String,
        dbname: String,
        keepalive_idle: Option<Duration>,
        ssl: bool,
    ) -> Result<Self, BuildError> {
        DatabaseClientInnerBuilder::build_with_params(host, port, user, password, dbname, keepalive_idle, ssl)
            .await
            .map(|client| Self { client })
    }

    /// # Panics
    /// # Errors
    pub async fn import_tables_from_file(&self, path: &str) -> Result<(), Error> {
        self.client
            .import_tables_from_file(path)
            .await
            .map_err(|error| Error::FileNotFound(format!("Error while load tables from file: {:?}", error)))
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    #[allow(clippy::cast_possible_truncation)]
    pub async fn add_extrinsic_realis(&self, response: &RealisEventType) -> Result<(), Error> {
        let status = Status::Got as u32;

        match response {
            RealisEventType::TransferNftToBsc(event) => {
                let value = serde_json::to_value(&event.token_id).unwrap();
                let types_nft = 2_u32;
                let block = event.block as u32;
                self.client
                    .client_pool
                    .get()
                    .await
                    .map_err(|error| DbPool(error))?
                    .execute(
                        "INSERT INTO extrinsics_realis(hash, block, \
                        from_account, to_account, value, type, status) \
                    VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        &[
                            &format!("{:?}", event.hash),
                            &block,
                            &event.from.to_string(),
                            &format!("{:?}", event.dest),
                            &value,
                            &types_nft,
                            &status,
                        ],
                    )
                    .await
                    .map_err(Error::Postgres)
                    .map(|_| ())
            }
            RealisEventType::TransferTokenToBsc(event) => {
                let value = serde_json::to_value(&event.amount.to_string()).unwrap();
                let types_tokens = 1_u32;
                let block = event.block as u32;
                self.client
                    .client_pool
                    .get()
                    .await
                    .map_err(|error| DbPool(error))?
                    .execute(
                        "INSERT INTO extrinsics_realis(hash, block, \
                        from_account, to_account, value, type, status) \
                    VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        &[
                            &format!("{:?}", event.hash),
                            &block,
                            &event.from.to_string(),
                            &format!("{:?}", event.to),
                            &value,
                            &types_tokens,
                            &status,
                        ],
                    )
                    .await
                    .map_err(Error::Postgres)
                    .map(|_| ())
            }
            RealisEventType::TransferTokenToRealisFail(_event) => Ok(()),
            RealisEventType::TransferNftToRealisFail(_event) => Ok(()),
        }
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    #[allow(clippy::cast_possible_truncation)]
    pub async fn add_extrinsic_bsc(&self, response: &BscEventType) -> Result<(), Error> {
        let status = Status::Got as u32;

        match response {
            BscEventType::TransferNftToRealis(event, ..) => {
                let value = serde_json::to_value(&event.token_id).unwrap();
                let types_nft = 2_u32;
                let block = event.block.unwrap().as_u32();
                self.client
                    .client_pool
                    .get()
                    .await
                    .map_err(|error| DbPool(error))?
                    .execute(
                        "INSERT INTO extrinsics_bsc(hash, block, \
                        from_account, to_account, value, type, status) \
                    VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        &[
                            &format!("{:?}", event.hash),
                            &block,
                            &format!("{:?}", event.from),
                            &event.dest.to_string(),
                            &value,
                            &types_nft,
                            &status,
                        ],
                    )
                    .await
                    .map_err(Error::Postgres)
                    .map(|_| ())
            }
            BscEventType::TransferTokenToRealis(event, ..) => {
                let value = serde_json::to_value(&event.amount.to_string()).unwrap();
                let types_tokens = 1_u32;
                let block: u32 = event.block.unwrap().as_u32();
                self.client
                    .client_pool
                    .get()
                    .await
                    .map_err(|error| DbPool(error))?
                    .execute(
                        "INSERT INTO extrinsics_bsc(hash, block, \
                        from_account, to_account, value, type, status) \
                    VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        &[
                            &format!("{:?}", event.hash),
                            &block,
                            &format!("{:?}", event.from),
                            &event.to.to_string(),
                            &value,
                            &types_tokens,
                            &status,
                        ],
                    )
                    .await
                    .map_err(Error::Postgres)
                    .map(|_| ())
            }
            BscEventType::TransferTokenToBscFail(_event) => Ok(()),
            BscEventType::TransferNftToBscFail(_event) => Ok(()),
        }
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn get_last_block_realis(&self) -> Result<BlockNumber, Error> {
        let block_number_batch = self
            .client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .query_one("SELECT max(block) FROM blocks_realis", &[])
            .await
            .map_err(Error::Postgres)?
            .try_get::<_, u32>(0)
            .map_err(Error::Postgres)
            .map(u64::from)?;
        Ok(block_number_batch)
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    #[allow(clippy::cast_possible_truncation)]
    pub async fn update_block_realis(&self, block: BlockNumber) -> Result<(), Error> {
        let block = block as u32;

        self.client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .execute(
                "INSERT INTO blocks_realis(block) \
                    VALUES ($1)",
                &[&block],
            )
            .await
            .map_err(Error::Postgres)?;

        Ok(())
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn get_last_block_bsc(&self) -> Result<BlockNumber, Error> {
        let block_number_batch = self
            .client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .query_one("SELECT max(block) FROM blocks_bsc", &[])
            .await
            .map_err(Error::Postgres)?
            .try_get::<_, u32>(0)
            .map_err(Error::Postgres)
            .map(u64::from)?;
        Ok(block_number_batch)
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn update_block_bsc(&self, block: Option<U64>) -> Result<(), Error> {
        let block = block.unwrap().as_u32();

        self.client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .execute(
                "INSERT INTO blocks_bsc(block) \
                    VALUES ($1)",
                &[&block],
            )
            .await
            .map_err(Error::Postgres)?;
        Ok(())
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn update_status_realis(&self, hash: &str, status: Status) -> Result<(), Error> {
        self.client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .execute(
                "UPDATE extrinsics_realis \
                SET status = $1 \
                WHERE hash=$2",
                &[&(status as u32), &hash],
            )
            .await
            .map(|_| ())
            .map_err(Error::Postgres)
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn update_status_bsc(&self, hash: &str, status: Status) -> Result<(), Error> {
        self.client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .execute(
                "UPDATE extrinsics_bsc \
                SET status = $1 \
                WHERE hash=$2",
                &[&(status as u32), &hash.to_string()],
            )
            .await
            .map(|_| ())
            .map_err(Error::Postgres)?;
        Ok(())
    }

    /// # Panics
    /// # Errors
    #[macro_retry]
    pub async fn add_raw_event(&self, raw_event: &RawEvent) -> Result<(), Error> {
        self.client
            .client_pool
            .get()
            .await
            .map_err(|error| DbPool(error))?
            .execute(
                "INSERT INTO undecoded_events(block, hash, data) \
            VALUES($1, $2, $3)",
                &[
                    &raw_event.block_number.unwrap().as_u32(),
                    &format!("{:?}", raw_event.hash),
                    &raw_event.data,
                ],
            )
            .await
            .map_err(Error::Postgres)
            .map(|_| ())
    }
}
