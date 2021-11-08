use db::Database;
use ethabi::ParamType;
use log::{error, info, warn};
use primitives::events::bsc::{BscEventType, TransferNftToRealis, TransferTokenToRealis};
use realis_primitives::TokenId;

use runtime::AccountId;
use serde::Deserialize;
use serde_json::Value;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;
use web3::{
    self,
    transports::WebSocket,
    types::{Transaction, H256},
    Web3,
};

#[derive(Clone)]
pub struct TxSender {
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
}

impl TxSender {
    pub async fn new(tx: Sender<BscEventType>, status: Arc<AtomicBool>) -> Self {
        Self { tx, status }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_tokens(&self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        let tx = web3.eth().transaction_receipt(transaction.hash).await.unwrap().unwrap();
        // TODO get from env
        let hash =
            H256::from_str("0xcd4959d4603f340036d296d8ab78401d37c53d963d84bf774509d2bebecf5702")
                .unwrap();

        for log in tx.logs.iter().filter(|log| log.topics.contains(&hash)) {
            info!(
                "{:?}",
                ethabi::decode(&[ParamType::String, ParamType::Uint(256)], &log.data.0)
            );
            match ethabi::decode(
                &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                &log.data.0,
            ) {
                Ok(info) => {
                    let account_from = info[2].clone().into_address().unwrap();
                    let json: Result<Value, serde_json::error::Error> =
                        serde_json::to_value(&info[0].to_string());
                    match json {
                        Ok(value) => {
                            let result: Result<AccountId, _> = Deserialize::deserialize(value);
                            match result {
                                Ok(account_id) => match info[1].clone().into_uint() {
                                    Some(amount) => {
                                        let amount = amount.as_u128();

                                        info!("Transaction hash: {:?}", transaction.hash);
                                        info!("{:?}", tx);
                                        let event = BscEventType::TransferTokenToRealis(
                                            TransferTokenToRealis {
                                                block: transaction.block_number,
                                                hash: transaction.hash,
                                                from: account_from,
                                                to: account_id,
                                                amount,
                                            },
                                        );
                                        match db.add_extrinsic_bsc(&event).await {
                                            Ok(()) => info!("Success add extrinsic in Database!"),
                                            Err(error) => {
                                                error!("Cannot add extrinsoc in Database: {:?}", error)
                                            }
                                        }
                                        match self.tx.send(event).await {
                                            Ok(()) => info!("Success send to realis-adapter!"),
                                            Err(error) => {
                                                self.status.store(false, Ordering::SeqCst);
                                                error!("Cannot send to realis-adapter: {:?}", error);
                                            }
                                        }
                                    }
                                    None => warn!("Cannot get amount!"),
                                },
                                Err(error) => {
                                    // TODO add event args to db
                                    warn!("Cannot deserialize Realis account: {:?}", error)
                                }
                            }
                        }
                        Err(error) => error!("Cannot serialize Realis account: {:?}", error),
                    }
                }
                Err(error) => {
                    error!("Decode token event: {:?}", error);
                }
            }
        }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_nft(&self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        let tx = web3.eth().transaction_receipt(transaction.hash).await.unwrap().unwrap();
        // FIXME magic hash string
        let hash =
            H256::from_str("0x50158efb7abc93588bff90584e6f7e94a75c3660da924b938aad8001afa5aa12")
                .unwrap();
        for log in tx.logs.iter().filter(|log| log.topics.contains(&hash)) {
            info!(
                "{:?}",
                ethabi::decode(
                    &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                    &log.data.0
                )
            );
            match ethabi::decode(
                &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                &log.data.0,
            ) {
                Ok(info) => {
                    let account_from = info[0].clone().into_address().unwrap();
                    let json: Value = serde_json::to_value(&info[1].to_string()).unwrap();
                    // FIXME remove this unwrap can cause to drop
                    let account_id: AccountId = Deserialize::deserialize(json).unwrap();
                    // FIXME remove this unwrap can cause to drop
                    let token_id = TokenId::from_str(&info[2].to_string()).unwrap();
                    info!("Transaction hash: {:?}", transaction.hash);
                    info!("{:?}", tx);
                    let event = BscEventType::TransferNftToRealis(TransferNftToRealis {
                        block: transaction.block_number,
                        hash: transaction.hash,
                        from: account_from,
                        dest: account_id,
                        token_id,
                    });
                    match db.add_extrinsic_bsc(&event).await {
                        Ok(()) => info!("Success add extrinsic in Database!"),
                        Err(error) => error!("Cannot add extrinsic in Database: {:?}", error),
                    }
                    match self.tx.send(event).await {
                        Ok(()) => info!("Success send to realis-adapter!"),
                        Err(error) => {
                            self.status.store(false, Ordering::SeqCst);
                            error!("Cannot send to realis-adapter: {:?}", error);
                        }
                    }
                }
                Err(error) => {
                    error!("Unable decode nft event: {:?}", error);
                }
            }
        }
    }
}