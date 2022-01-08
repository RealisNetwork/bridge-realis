use primitives::{types::BlockNumber, Error};

use primitives::{
    db::Status,
    events::{bsc::BscEventType, realis::RealisEventType},
    types::RawEvent,
};
use rust_lib::healthchecker::HealthChecker;
use web3::ethabi::ethereum_types::U64;
use rust_lib::inner_db::client_inner::DatabaseClientInner;
use rust_lib::inner_db::client_inner_builder::DatabaseClientInnerBuilder;

pub struct Database {
    client: DatabaseClientInner,
}

impl Database {
    /// # Panics
    /// # Errors
    pub async fn new(host: &str, port: &str, user: &str, password: &str, dbname: &str, ssl: bool, health: HealthChecker,) -> Result<Self, tokio_postgres::Error> {
        DatabaseClientInnerBuilder::build_with_params(host, port, user, password, dbname, ssl, health).await
            .map(|client| Self {client})
    }

    /// # Panics
    /// # Errors
    pub async fn still_alive(&self) -> Result<(), Error> {
        self.client.still_alive().await.map_err(|_| Error::Disconnected)
    }

    /// # Panics
    /// # Errors
    pub async fn import_tables_from_file(&self, path: &str) -> Result<(), Error> {
        self.still_alive().await?;

        self.client.import_tables_from_file(path).await.map_err(|error| {
            Error::FileNotFound(format!("Error while load tables from file: {:?}", error))
        })
    }

    /// # Panics
    /// # Errors
    #[allow(clippy::cast_possible_truncation)]
    pub async fn add_extrinsic_realis(&self, response: &RealisEventType) -> Result<(), Error> {
        self.still_alive().await?;

        let status = Status::Got as u32;

        match response {
            RealisEventType::TransferNftToBsc(event) => {
                let value = serde_json::to_value(&event.token_id).unwrap();
                let types_nft = 2_u32;
                let block = event.block as u32;
                self.client
                    .client
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
                    .client
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
    #[allow(clippy::cast_possible_truncation)]
    pub async fn add_extrinsic_bsc(&self, response: &BscEventType) -> Result<(), Error> {
        self.still_alive().await?;
        let status = Status::Got as u32;

        match response {
            BscEventType::TransferNftToRealis(event, ..) => {
                let value = serde_json::to_value(&event.token_id).unwrap();
                let types_nft = 2_u32;
                let block = event.block.unwrap().as_u32();
                self.client
                    .client
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
                    .client
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
    pub async fn get_last_block_realis(&self) -> Result<BlockNumber, Error> {
        self.still_alive().await?;

        let block_number_batch = self
            .client
            .client
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
    #[allow(clippy::cast_possible_truncation)]
    pub async fn update_block_realis(&self, block: BlockNumber) -> Result<(), Error> {
        self.still_alive().await?;

        let block = block as u32;

        self.client
            .client
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
    pub async fn get_last_block_bsc(&self) -> Result<BlockNumber, Error> {
        self.still_alive().await?;

        let block_number_batch = self
            .client
            .client
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
    pub async fn update_block_bsc(&self, block: Option<U64>) -> Result<(), Error> {
        self.still_alive().await?;

        let block = block.unwrap().as_u32();

        self.client
            .client
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
    pub async fn update_status_realis(&self, hash: &str, status: Status) -> Result<(), Error> {
        self.still_alive().await?;
        self.client
            .client
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
    pub async fn update_status_bsc(&self, hash: &str, status: Status) -> Result<(), Error> {
        self.still_alive().await?;

        self.client
            .client
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
    pub async fn add_raw_event(&self, raw_event: RawEvent) -> Result<(), Error> {
        self.still_alive().await?;

        self.client
            .client
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
