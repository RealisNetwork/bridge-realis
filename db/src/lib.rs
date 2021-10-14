use primitives::{events::EventType, types::BlockNumber, Error};

use log::{error, trace};
use postgres::NoTls;
use rawsql::{self, Loader};
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio_postgres::Client;

pub struct Database {
    client: Client,
    has_err: Arc<AtomicBool>,
}

impl Database {
    /// # Panics
    /// # Errors
    pub async fn new(params: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(params, NoTls).await?;
        let has_err = Arc::new(AtomicBool::new(false));
        tokio::spawn({
            let has_err = Arc::clone(&has_err);
            async move {
                if connection.await.is_err() {
                    has_err.store(true, Ordering::Release);
                }
            }
        });

        Ok(Database { client, has_err })
    }

    /// # Panics
    /// # Errors
    pub async fn still_alive(&self) -> Result<(), Error> {
        if self.has_err.load(Ordering::Acquire) {
            Err(Error::Disconnected)
        } else {
            Ok(())
        }
    }

    /// # Panics
    /// # Errors
    pub async fn add_extrinsic(&self, response: &EventType) -> Result<(), Error> {
        self.still_alive().await?;

        match response {
            EventType::TransferTokenToBscSuccess(event, hash, block_number)
            | EventType::TransferTokenToBscError(event, hash, block_number) => {
                let block_number = u32::try_from(*block_number).unwrap();

                self.client
                    .execute(
                        "INSERT INTO extrinsics(from, to, amount, hash, block_number) \
                    VALUES ($1, $2, $3) \
                    ON CONFLICT(request_id) DO UPDATE \
                        SET hash = excluded.hash,\
                            block_number = excluded.block_number",
                        &[&event.from, &format!("{:?}", hash), &block_number],
                    )
                    .await
                    .map_err(Error::Postgres)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// # Panics
    /// # Errors
    pub async fn import_tables_from_file(&self, path: &str) -> Result<(), Error> {
        self.still_alive().await?;

        let queries = Loader::get_queries_from(path)
            .map_err(|_| Error::FileNotFound(String::from(path)))?
            .queries;

        let mut queries = queries.iter().collect::<Vec<(&String, &String)>>();

        queries.sort();

        for query in queries {
            match self.client.execute(query.1.as_str(), &[]).await {
                Ok(_value) => trace!("Successful send query!"),
                Err(error) => error!("Cannot send query: {:?}", error),
            }
        }

        Ok(())
    }

    /// # Panics
    /// # Errors
    pub async fn contains_extrinsic(&self, event: &EventType) -> Result<bool, Error> {
        self.still_alive().await?;

        match event {
            EventType::TransferTokenToBscSuccess(event, _, _)
            | EventType::TransferTokenToBscError(event, _, _) => self
                .client
                .query_one(
                    "SELECT COUNT(*) \
                    FROM extrinsics_batch \
                    WHERE request_id = $1",
                    &[&event.id],
                )
                .await
                .map_err(Error::Postgres)?
                .try_get::<_, u32>(0)
                .map_err(Error::Postgres)
                .map(|value| value != 0),
            _ => {}
        }
    }

    /// # Panics
    /// # Errors
    pub async fn get_last_block(&self) -> Result<BlockNumber, Error> {
        self.still_alive().await?;

        let block_number_batch = self
            .client
            .query_one("SELECT max(block_number) FROM extrinsics_batch", &[])
            .await
            .map_err(Error::Postgres)?
            .try_get::<_, u32>(0)
            .map_err(Error::Postgres)
            .map(u64::from)?;
        Ok(block_number_batch)
    }
}