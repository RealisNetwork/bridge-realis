mod connection_builder;

use crate::connection_builder::ConnectionBuilder;
use tokio::{select, sync::mpsc::Receiver};

use log::{error, info};
use primitives::Error;
use rust_lib::healthchecker::HealthChecker;
use secp256k1::SecretKey;

use db::Database;
use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use primitives::db::Status;
use web3::{transports::WebSocket, Web3};
use primitives::events::realis::RealisEventType;
use primitives::events::traits::Event;

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
                                // TODO send to realis-adapter
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

        let (contract, (func, params)) = match request {
            RealisEventType::TransferNftToBsc(request, ..) => {
                (
                    ConnectionBuilder::nft(connection, &self.nft_contract_address).await?,
                    request.get_binance_call()
                )
            }
            RealisEventType::TransferTokenToBsc(request, ..) => {
                (
                    ConnectionBuilder::token(connection, &self.token_contract_address).await?,
                    request.get_binance_call()
                )
            }
        };

        let success_contract = contract.signed_call_with_confirmations(
                &func,
                params,
                // TODO get this options from blockchain data
                web3::contract::Options::default(),
                // TODO check this
                1,
                &self.master_key,
            )
            .await
            .map_err(Error::Web3)
            .map(|_| ());

        let status = match success_contract {
            Ok(_) => Status::Success,
            Err(_) => Status::Error,
        };

        match self
            .db
            .update_status_realis(&request.get_hash().to_string(), status)
            .await
        {
            Ok(_) => {
                info!("Update realis status InProgress");
            }
            Err(error) => {
                error!("Update realis status send error: {:?}", error);
            }
        }

        success_contract
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
