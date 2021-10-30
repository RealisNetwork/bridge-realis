use db::Database;
use ethabi::ParamType;
use log::{error, info};
use primitives::events::{BscEventType, TransferNftToRealis, TransferTokenToRealis};
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
    futures::StreamExt,
    transports::WebSocket,
    types::{Address, Transaction},
    Web3,
};

#[derive(Clone)]
pub struct BlockListener {
    url: String,
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
}

impl BlockListener {
    /// # Errors
    #[must_use]
    pub fn new(url: String, tx: Sender<BscEventType>, status: Arc<AtomicBool>) -> Self {
        Self { url, tx, status }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn listen(&self, db: Arc<Database>) {
        let ws = web3::transports::WebSocket::new(&self.url).await.unwrap();
        let web3 = web3::Web3::new(ws.clone());
        // Stream
        let mut sub = web3.eth_subscribe().subscribe_new_heads().await.unwrap();

        info!("Got subscription id: {:?}", sub.id());

        while let Some(value) = sub.next().await {
            let block = value.unwrap();
            // &db.update_block_bsc(block.number).await;
            let some = web3
                .eth()
                .block_with_txs(web3::types::BlockId::Hash(block.hash.unwrap()))
                .await
                .unwrap()
                .unwrap();

            for transaction in some.transactions {
                match transaction.to {
                    Some(account) => {
                        if account == Address::from_str("0x1c43b4253c33d246ad27e710d949a8d8b62a2c73").unwrap() {
                            self.clone().send_tokens(transaction, web3.clone(), &db).await;
                        } else if account
                            == Address::from_str("0xc2f5Fb3eEFE324B263DfF8cDf6d0113ae2B6B19E").unwrap()
                        {
                            self.clone().send_nft(transaction, web3.clone(), &db).await;
                        }
                    }
                    None => (),
                }
            }
        }
        sub.unsubscribe().await.unwrap();
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_tokens(self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                if account_from != Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap() {
                    info!("[Transactions] - {:?}", transaction);
                    info!("Transaction hash: {:?}", transaction.hash);

                    let tx = web3.eth().transaction_receipt(transaction.hash).await.unwrap().unwrap();
                    info!("{:?}", tx);
                    for log in tx.logs {
                        info!(
                            "{:?}",
                            ethabi::decode(&[ParamType::String, ParamType::Uint(256)], &log.data.0)
                        );
                        let info = ethabi::decode(
                            &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                            &log.data.0,
                        )
                        .unwrap();
                        let json: Result<Value, serde_json::error::Error> =
                            serde_json::to_value(&info[0].to_string());
                        match json {
                            Ok(value) => {
                                let account_id: AccountId = Deserialize::deserialize(value).unwrap();
                                let amount = info[1].clone().into_uint().unwrap().as_u128();

                                info!("{:?}", account_id);
                                let event = BscEventType::TransferTokenToRealisSuccess(
                                    TransferTokenToRealis {
                                        block: transaction.block_number,
                                        hash: transaction.hash,
                                        from: account_from,
                                        to: account_id,
                                        amount,
                                    },
                                    transaction.hash,
                                    transaction.block_number,
                                );
                                match db.add_extrinsic_bsc(&event).await {
                                    Ok(()) => info!("Success add extrinsic in Database!"),
                                    Err(error) => error!("Cannot add extrinsoc in Database: {:?}", error),
                                }
                                match self.tx.send(event).await {
                                    Ok(()) => info!("Success send to realis-adapter!"),
                                    Err(error) => {
                                        self.status.store(false, Ordering::SeqCst);
                                        error!("Cannot send to realis-adapter: {:?}", error);
                                    }
                                }
                            }
                            Err(error) => error!("Cannot parse Realis account: {:?}", error),
                        }
                    }
                }
            }
            None => (),
        }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_nft(self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                if account_from != Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap() {
                    info!("[Transactions] - {:?}", transaction);
                    info!("Transaction hash: {:?}", transaction.hash);

                    let tx = web3.eth().transaction_receipt(transaction.hash).await.unwrap().unwrap();
                    info!("{:?}", tx);
                    for log in tx.logs {
                        info!(
                            "{:?}",
                            ethabi::decode(&[ParamType::String, ParamType::Uint(256)], &log.data.0)
                        );
                        let info = ethabi::decode(
                            &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                            &log.data.0,
                        )
                        .unwrap();
                        let json: Value = serde_json::from_str(&info[0].to_string()).unwrap();
                        let account_id: AccountId = Deserialize::deserialize(json).unwrap();
                        let token_id = TokenId::from_str(&info[0].to_string()).unwrap();
                        info!("{:?}", account_id);
                        let event = BscEventType::TransferNftToRealisSuccess(
                            TransferNftToRealis {
                                block: transaction.block_number,
                                hash: transaction.hash,
                                from: account_from,
                                dest: account_id,
                                token_id,
                            },
                            transaction.hash,
                            transaction.block_number,
                        );
                        match db.add_extrinsic_bsc(&event).await {
                            Ok(()) => info!("Success add extrinsic in Database!"),
                            Err(error) => error!("Cannot add extrinsoc in Database: {:?}", error),
                        }
                        match self.tx.send(event).await {
                            Ok(()) => info!("Success send to realis-adapter!"),
                            Err(error) => {
                                self.status.store(false, Ordering::SeqCst);
                                error!("Cannot send to realis-adapter: {:?}", error);
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }
}
