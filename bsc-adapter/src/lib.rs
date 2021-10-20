mod connection_builder;

use crate::connection_builder::ConnectionBuilder;

use db::Database;
use primitives::{events::EventType, Error, Status};

use log::{error, info, warn};
use secp256k1::SecretKey;
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use web3::{transports::WebSocket, types::U256, Web3};

pub struct BinanceHandler {
    rx: Receiver<EventType>,
    tx_responder: Sender<Value>,
    tx_rollback: Sender<EventType>,
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
        rx: Receiver<EventType>,
        tx_responder: Sender<Value>,
        tx_rollback: Sender<EventType>,
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
            tx_responder,
            tx_rollback,
            connection_builder,
            token_contract_address,
            nft_contract_address,
            status,
            master_key,
            db,
        }
    }

    pub async fn handle(mut self) {
        // TODO check handle still_alive status
        while let Some(request) = self.rx.recv().await {
            match self.execute(&request).await {
                Ok(_) => {
                    info!("Success send transaction to BSC!");
                }
                Err(error) => {
                    error!("Cannot send transaction {:?}", error);
                    self.status.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    async fn execute(&mut self, request: &EventType) -> Result<(), Error> {
        let connection = self.connect().await?;

        info!("Start send transaction");

        // let gas_price = connection.eth().gas_price().await.unwrap();

        match request {
            EventType::TransferNftToBscSuccess(request, ..) => {
                let contract =
                    ConnectionBuilder::nft(connection, &self.nft_contract_address)
                        .await?;

                let token_id =
                    U256::from_dec_str(&request.token_id.to_string()).unwrap();

                contract
                    .signed_call_with_confirmations(
                        // TODO check this
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
                    .map(|_| ())
            }
            EventType::TransferTokenToBscSuccess(request, ..) => {
                let contract = ConnectionBuilder::token(
                    connection,
                    &self.token_contract_address,
                )
                .await?;

                let amount = web3::types::U128::from(request.amount);

                contract
                    .signed_call_with_confirmations(
                        // TODO check this
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
                    .map(|_| ())
            }
            EventType::TransferTokenToBscError(..)
            | EventType::TransferNftToBscError(..) => Ok(()),
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
