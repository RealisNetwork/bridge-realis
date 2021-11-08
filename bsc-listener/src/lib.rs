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
    futures::StreamExt,
    transports::WebSocket,
    types::{Address, BlockNumber, Transaction, H256},
    Web3,
};

pub struct BlockListener {
    web3: Web3<WebSocket>,
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
    db: Arc<Database>,
    owner: Address,
    token_contract: Address,
    nft_contract: Address,
}

impl BlockListener {
    /// # Errors
    /// # Panics
    pub async fn new(url: String, tx: Sender<BscEventType>, status: Arc<AtomicBool>, db: Arc<Database>) -> Self {
        let ws = web3::transports::WebSocket::new(&url).await.unwrap();
        let web3 = web3::Web3::new(ws);
        Self { web3, tx, status, db }
    }

    /// # Errors
    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen_with_restore(&mut self, from: u64) {
        warn!("Start restore BSC!!!");
        let block_number = self.web3.eth().block_number().await.unwrap().as_u64();
        for number in from..block_number {
            warn!("Start restore!!!");
            let block = self
                .web3
                .eth()
                .block(web3::types::BlockId::Number(BlockNumber::from(number)))
                .await
                .unwrap()
                .unwrap();
            let some = self
                .web3
                .eth()
                .block_with_txs(web3::types::BlockId::Hash(block.hash.unwrap()))
                .await
                .unwrap()
                .unwrap();

            let tx_sender = TxSender::new(self.tx.clone(), self.status.clone()).await;
            for transaction in some.transactions {
                if let Some(account) = transaction.to {
                    // FIXME magic address string
                    if account == Address::from_str("0x1f2dc9face361851ef1d2adeb15cbeb206ec104c").unwrap() {
                        tx_sender
                            .clone()
                            .send_tokens(transaction, self.web3.clone(), &self.db)
                            .await;
                        // FIXME magic address string
                    } else if account == Address::from_str("0x0875cb9090010e2844aefA88c879a8bBda8d70C8").unwrap() {
                        tx_sender
                            .clone()
                            .send_nft(transaction, self.web3.clone(), &self.db)
                            .await;
                    }
                }
            }
        }
        self.listen().await;
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn listen(&mut self) {
        // Stream
        let mut sub = self.web3.eth_subscribe().subscribe_new_heads().await.unwrap();

        info!("Got subscription id: {:?}", sub.id());

        while let Some(value) = sub.next().await {
            let block = value.unwrap();
            match self.db.update_block_bsc(block.number).await {
                Ok(_) => {
                    info!("Success add binance block to database");
                }
                Err(error) => {
                    error!("Can't add binance block with error: {:?}", error);
                }
            }
            let some = self
                .web3
                .eth()
                .block_with_txs(web3::types::BlockId::Hash(block.hash.unwrap()))
                .await
                .unwrap()
                .unwrap();

            let tx_sender = TxSender::new(self.tx.clone(), self.status.clone()).await;
            for transaction in some.transactions {
                match transaction.to {
                    Some(account) => {
                        // FIXME magic address string
                        if account == Address::from_str("0x1f2dc9face361851ef1d2adeb15cbeb206ec104c").unwrap() {
                            tx_sender
                                .clone()
                                .send_tokens(transaction, self.web3.clone(), &self.db)
                                .await;
                            // FIXME magic address string
                        } else if account
                            == Address::from_str("0x0875cb9090010e2844aefA88c879a8bBda8d70C8").unwrap()
                        {
                            tx_sender
                                .clone()
                                .send_nft(transaction, self.web3.clone(), &self.db)
                                .await;
                        }
                    }
                    None => (),
                }
            }
        }
        sub.unsubscribe().await.unwrap();
    }

    async fn execute(&self, transaction: Transaction) {
        if let Some(account) = transaction.to {
            let tx_sender = TxSender::new(self.tx.clone(), self.status.clone(), self.owner).await;

            if account == self.token_contract {
                tx_sender
                    .send_tokens(transaction, self.web3.clone(), &self.db)
                    .await;
            } else if account == self.nft_contract {
                tx_sender
                    .send_nft(transaction, self.web3.clone(), &self.db)
                    .await;
            }
        }
    }
}

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
    pub async fn send_tokens(self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                // FIXME magic address string
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
                        match ethabi::decode(
                            &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                            &log.data.0,
                        ) {
                            Ok(info) => {
                                let json: Result<Value, serde_json::error::Error> =
                                    serde_json::to_value(&info[0].to_string());
                                match json {
                                    Ok(value) => {
                                        // FIXME remove this unwrap can cause to drop
                                        let account_id: AccountId = Deserialize::deserialize(value).unwrap();
                                        // FIXME remove this unwrap can cause to drop
                                        let amount = info[1].clone().into_uint().unwrap().as_u128();

                                        info!("{:?}", account_id);
                                        let event = BscEventType::TransferTokenToRealis(TransferTokenToRealis {
                                            block: transaction.block_number,
                                            hash: transaction.hash,
                                            from: account_from,
                                            to: account_id,
                                            amount,
                                        });
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
                            Err(error) => {
                                error!("Decode token event: {:?}", error);
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_nft(&self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                // FIXME magic address string
                if account_from != Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap() {
                    info!("[Transactions] - {:?}", transaction);
                    info!("Transaction hash: {:?}", transaction.hash);

                    let tx = web3.eth().transaction_receipt(transaction.hash).await.unwrap().unwrap();
                    info!("{:?}", tx);
                    // FIXME magic hash string
                    let hash =
                        H256::from_str("0x50158efb7abc93588bff90584e6f7e94a75c3660da924b938aad8001afa5aa12")
                            .unwrap();
                    for log in tx.logs {
                        for topic in log.topics {
                            if topic == hash {
                                warn!("{:?}", topic);
                                info!(
                                    "{:?}",
                                    ethabi::decode(
                                        &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                                        &log.data.0
                                    )
                                );
                                // TODO get rid of this unwrap
                                match ethabi::decode(
                                    &[ParamType::Address, ParamType::String, ParamType::Uint(256)],
                                    &log.data.0,
                                ) {
                                    Ok(info) => {
                                        let json: Value = serde_json::to_value(&info[1].to_string()).unwrap();
                                        warn!("{:?}", json);
                                        // FIXME remove this unwrap can cause to drop
                                        let account_id: AccountId = Deserialize::deserialize(json).unwrap();
                                        // FIXME remove this unwrap can cause to drop
                                        let token_id = TokenId::from_str(&info[2].to_string()).unwrap();
                                        info!("{:?}", account_id);
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
                                        error!("Decode nft event: {:?}", error);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }
}
