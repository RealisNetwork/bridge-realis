mod connection_builder;

use crate::connection_builder::ConnectionBuilder;
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
};

use log::{error, info};
use primitives::Error;
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

use primitives::{
    db::Status,
    events::{bsc::BscEventType, realis::RealisEventType, traits::Event},
};
use web3::{
    contract::{tokens::Tokenize, Contract},
    transports::WebSocket,
    types::{TransactionReceipt, U64},
    Web3,
};

#[allow(dead_code)]
pub struct BinanceHandler {
    rx: Receiver<RealisEventType>,
    tx: Sender<BscEventType>,
    connection_builder: ConnectionBuilder,
    token_contract_address: String,
    nft_contract_address: String,
    status: Arc<AtomicBool>,
    master_key: SecretKey,
    db: Arc<Database>,
}

impl BinanceHandler {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    /// # Panics
    pub fn new(
        rx: Receiver<RealisEventType>,
        tx: Sender<BscEventType>,
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
            tx,
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
                                let rollback_request = match request {
                                    RealisEventType::TransferNftToBsc(request, ..) => {
                                        Some(BscEventType::TransferNftToBscFail(request))
                                    }
                                    RealisEventType::TransferTokenToBsc(request, ..) => {
                                        Some(BscEventType::TransferTokenToBscFail(request))
                                    }
                                    // If rollback request fail
                                    _ => None
                                };
                                if let Some(rollback_request) = rollback_request {
                                    error!("Extrinsic execute: {:?}", error);
                                    if let Err(error) = self.tx.send(rollback_request).await {
                                        error!("[BSC Adapter] - send error: {:?}", error);
                                        self.status.store(false, Ordering::SeqCst);
                                    }
                                } else {
                                    error!("Rollback fail: {:?}", error);
                                    self.status.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute(&self, request: &RealisEventType) -> Result<(), Error> {
        let connection = self.connect().await?;

        info!("Start send transaction");

        match request {
            RealisEventType::TransferNftToBsc(event) => {
                self.process(
                    event,
                    ConnectionBuilder::nft(connection, &self.nft_contract_address).await?,
                )
                .await
            }
            RealisEventType::TransferTokenToBsc(event) => {
                self.process(
                    event,
                    ConnectionBuilder::token(connection, &self.token_contract_address).await?,
                )
                .await
            }
            RealisEventType::TransferNftToRealisFail(event) => {
                self.rollback(
                    event,
                    ConnectionBuilder::nft(connection, &self.nft_contract_address).await?,
                )
                .await
            }
            RealisEventType::TransferTokenToRealisFail(event) => {
                self.rollback(
                    event,
                    ConnectionBuilder::token(connection, &self.token_contract_address).await?,
                )
                .await
            }
        }
    }

    async fn connect(&self) -> Result<Web3<WebSocket>, Error> {
        for _ in 0..10 {
            if let Ok(connection) = self.connection_builder.connect().await {
                return Ok(connection);
            }
        }

        Err(Error::Custom(String::from("Can't connect to binance!")))
    }

    async fn process(&self, event: &impl Event, contract: Contract<WebSocket>) -> Result<(), Error> {
        self
            .db
            .update_status_realis(&event.get_hash(), Status::InProgress)
            .await?;

        let (func, params) = event.get_binance_call();

        let result = self
            .send_to_blockchain(
                contract,
                &func,
                (params[0].clone(), params[1].clone(), params[2].clone()),
            )
            .await;

        if let Err(error) = self
            .db
            .update_status_realis(
                &event.get_hash(),
                result.as_ref().map(|_| Status::Success).unwrap_or(Status::Error),
            )
            .await {
            error!("[BSC Adapter] - logging status to db: {:?}", error);
            self.status.store(false, Ordering::SeqCst);
        }

        result
    }

    async fn rollback(&self, event: &impl Event, contract: Contract<WebSocket>) -> Result<(), Error> {
        let (func, params) = event.get_binance_call();

        let result = if params.len() == 3 {
            self.send_to_blockchain(
                contract,
                &func,
                (params[0].clone(), params[1].clone(), params[2].clone()),
            )
            .await
        } else {
            self.send_to_blockchain(contract, &func, (params[0].clone(), params[1].clone()))
                .await
        };

        if let Err(error) = self
            .db
            .update_status_realis(
                &event.get_hash(),
                result
                    .as_ref()
                    .map(|_| Status::RollbackSuccess)
                    .unwrap_or(Status::RollbackError),
            )
            .await {
            error!("[BSC Adapter] - logging status to db: {:?}", error);
            self.status.store(false, Ordering::SeqCst);
        }

        result
    }

    async fn send_to_blockchain(
        &self,
        contract: Contract<WebSocket>,
        func: &str,
        params: impl Tokenize,
    ) -> Result<(), Error> {
        let receipt = contract
            .signed_call_with_confirmations(
                func,
                params,
                web3::contract::Options::default(),
                1,
                &self.master_key,
            )
            .await
            .map_err(Error::Web3)?;

        Self::check_extrinsic(&receipt)
    }

    fn check_extrinsic(receipt: &TransactionReceipt) -> Result<(), Error> {
        if receipt.status == Some(U64::from(1)) {
            Ok(())
        } else {
            Err(Error::Custom(String::from("No confirmation found!")))
        }
    }
}
