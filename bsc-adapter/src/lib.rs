mod connection_builder;

use crate::connection_builder::ConnectionBuilder;
use tokio::{select, sync::mpsc::Receiver};

use log::{error, info};
use primitives::{events::RealisEventType, Error};
use rust_lib::healthchecker::HealthChecker;
use secp256k1::SecretKey;

use db::Database;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use primitives::db::Status;
use web3::{transports::WebSocket, types::U256, Web3};

#[allow(dead_code)]
pub struct BinanceHandler {
    rx: Receiver<RealisEventType>,
    connection_builder: ConnectionBuilder,
    token_contract_address: String,
    nft_contract_address: String,
    status: Arc<AtomicBool>,
    master_key: SecretKey,
    db: Arc<Database>,
}

impl BinanceHandler {
    #[must_use]
    /// # Panics
    pub fn new(
        rx: Receiver<RealisEventType>,
        status: Arc<AtomicBool>,
        url: &str,
        token_contract_address: String,
        nft_contract_address: String,
        master_key: &str,
        db: Arc<Database>,
    ) -> Self {
        let connection_builder = ConnectionBuilder::new(url);
        let master_key = SecretKey::from_str(master_key).unwrap();

        Self {
            rx,
            connection_builder,
            token_contract_address,
            nft_contract_address,
            status,
            master_key,
            db,
        }
    }

    /// # Panics
    /// # Errors
    pub async fn handle(mut self) {
        loop {
            select! {
                () = HealthChecker::is_alive(Arc::clone(&self.status)) => break,
                option = self.rx.recv() => {
                    if let Some(request) = option {
                        match self.execute(&request).await {
                            Ok(_) => {
                                info!("Success send transaction to Realis!");
                            }
                            Err(error) => {
                                error!("Cannot send transaction {:?}", error);
                                self.status.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute(&mut self, request: &RealisEventType) -> Result<(), Error> {
        let connection = self.connect().await?;

        info!("Start send transaction");

        // let gas_price = connection.eth().gas_price().await.unwrap();

        match request {
            RealisEventType::TransferNftToBsc(request, ..) => {
                let contract = ConnectionBuilder::nft(connection, &self.nft_contract_address).await?;

                let token_id = U256::from_dec_str(&request.token_id.to_string()).unwrap();

                let success_contract = contract
                    .signed_call_with_confirmations(
                        "safeMint",
                        (request.dest, token_id),
                        // TODO get this options from blockchain data
                        web3::contract::Options::default(),
                        // TODO check this
                        1,
                        &self.master_key,
                    )
                    .await
                    .map_err(Error::Web3)
                    .map(|_| ());
                match self.db.update_status_realis(&request.hash, Status::InProgress).await {
                    Ok(_) => {
                        info!("Success realis status InProgress");
                    }
                    Err(error) => {
                        error!("Update realis status send error: {:?}", error);
                    }
                }
                // TODO CHECK update status to in progress, if got hash, update to complete
                success_contract
            }
            RealisEventType::TransferTokenToBsc(request, ..) => {
                let contract = ConnectionBuilder::token(connection, &self.token_contract_address).await?;

                let amount = web3::types::U128::from(request.amount);

                let success_contract = contract
                    .signed_call_with_confirmations(
                        "transferFromRealis",
                        (request.from.to_string(), request.to, amount),
                        // TODO get this options from blockchain data
                        web3::contract::Options::default(),
                        // TODO check this
                        1,
                        &self.master_key,
                    )
                    .await
                    .map_err(Error::Web3)
                    .map(|_| ());
                match self.db.update_status_realis(&request.hash, Status::InProgress).await {
                    Ok(_) => {
                        info!("Update realis status InProgress");
                    }
                    Err(error) => {
                        error!("Update realis status send error: {:?}", error);
                    }
                }
                // TODO CHECK update status to in progress, if got hash, update to complete
                success_contract
            }
        }
    }

    async fn connect(&mut self) -> Result<Web3<WebSocket>, Error> {
        for _ in 0..10 {
            if let Ok(connection) = self.connection_builder.connect().await {
                return Ok(connection);
            }
        }

        Err(Error::Custom(String::from("Can't connect to binance!")))
    }
}
