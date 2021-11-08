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
    pub async fn new(
        url: String,
        tx: Sender<BscEventType>,
        status: Arc<AtomicBool>,
        db: Arc<Database>,
        owner: &str,
        token_contract: &str,
        nft_contract: &str,
    ) -> Self {
        // TODO get rid of unwrap
        let ws = web3::transports::WebSocket::new(&url).await.unwrap();
        let web3 = web3::Web3::new(ws);
        // TODO get rid of unwraps
        let owner = Address::from_str(owner).unwrap();
        let token_contract = Address::from_str(token_contract).unwrap();
        let nft_contract = Address::from_str(nft_contract).unwrap();

        Self {
            web3,
            tx,
            status,
            db,
            owner,
            token_contract,
            nft_contract,
        }
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

            for transaction in some.transactions {
                self.execute(transaction).await;
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

            for transaction in some.transactions {
                self.execute(transaction).await;
            }
        }
        sub.unsubscribe().await.unwrap();
    }

    async fn execute(&self, transaction: Transaction) {
        if let Some(account) = transaction.to {
            let tx_sender = TxSender::new(self.tx.clone(), self.status.clone(), self.owner).await;

            if account == self.token_contract {
                tx_sender.send_tokens(transaction, self.web3.clone(), &self.db).await;
            } else if account == self.nft_contract {
                tx_sender.send_nft(transaction, self.web3.clone(), &self.db).await;
            }
        }
    }
}

// TODO move to separate file
#[derive(Clone)]
pub struct TxSender {
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
    owner: Address,
}

impl TxSender {
    pub async fn new(tx: Sender<BscEventType>, status: Arc<AtomicBool>, owner: Address) -> Self {
        Self { tx, status, owner }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_tokens(&self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                if account_from != self.owner {
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
            }
            None => (),
        }
    }

    /// # Panics
    #[allow(clippy::single_match)]
    pub async fn send_nft(&self, transaction: Transaction, web3: Web3<WebSocket>, db: &Database) {
        match transaction.from {
            Some(account_from) => {
                if account_from != self.owner {
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
            None => (),
        }
    }
}
