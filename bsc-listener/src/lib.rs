mod event_parser;
use crate::event_parser::{EventParser, ParseError};

use db::Database;
use primitives::events::bsc::{BscEventType};

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool},
        Arc,
    },
};
use std::sync::atomic::Ordering;
use ethabi::ethereum_types::H256;
use tokio::sync::mpsc::Sender;
use web3::{
    self,
    futures::StreamExt,
    transports::WebSocket,
    types::{Address, BlockNumber, Transaction},
    Web3,
};
use log::{error, info, warn};
use primitives::Error;


pub struct BlockListener {
    web3: Web3<WebSocket>,
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
    db: Arc<Database>,
    token_contract: Address,
    nft_contract: Address,
    token_topic: H256,
    nft_topic: H256,
}

impl BlockListener {
    /// # Errors
    /// # Panics
    pub async fn new(
        url: String,
        tx: Sender<BscEventType>,
        status: Arc<AtomicBool>,
        db: Arc<Database>,
        token_contract: &str,
        nft_contract: &str,
    ) -> Self {
        // TODO get rid of unwrap
        let ws = web3::transports::WebSocket::new(&url).await.unwrap();
        let web3 = web3::Web3::new(ws);
        // TODO get rid of unwraps
        let token_contract = Address::from_str(token_contract).unwrap();
        let nft_contract = Address::from_str(nft_contract).unwrap();

        // TODO get from env
        let token_topic = H256::from_str("0xcd4959d4603f340036d296d8ab78401d37c53d963d84bf774509d2bebecf5702")
            .unwrap();
        // TODO get from env
        let nft_topic = H256::from_str("0xcd4959d4603f340036d296d8ab78401d37c53d963d84bf774509d2bebecf5702")
            .unwrap();

        Self {
            web3,
            tx,
            status,
            db,
            token_contract,
            nft_contract,
            token_topic,
            nft_topic,
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
                self.process(transaction).await;
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
            let block_header = value.unwrap();
            match self.db.update_block_bsc(block_header.number).await {
                Ok(_) => {
                    info!("Success add binance block to database");
                }
                Err(error) => {
                    //TODO maybe should drop?
                    error!("Can't add binance block with error: {:?}", error);
                }
            }
            let block = self
                .web3
                .eth()
                .block_with_txs(web3::types::BlockId::Hash(block_header.hash.unwrap()))
                .await
                .unwrap()
                .unwrap();

            for transaction in block.transactions {
                self.process(transaction).await;
            }
        }
        sub.unsubscribe().await.unwrap();
    }

    async fn process(&self, transaction: Transaction) {
        if let Some(account) = transaction.to {
            if account == self.token_contract {
                if let Ok(Some(receipt)) = self.web3.eth().transaction_receipt(transaction.hash).await {
                    let events = event_parser::TokenParser::parse(receipt, &self.token_topic);
                    self.execute(events).await;
                }
            } else if account == self.nft_contract {
                if let Ok(Some(receipt)) = self.web3.eth().transaction_receipt(transaction.hash).await {
                    let events = event_parser::NftParser::parse(receipt, &self.nft_topic);
                    self.execute(events).await;
                }
            }
        }
    }

    async fn execute(&self, events: Vec<Result<BscEventType, ParseError>>) {
        for event in events {
            match event {
                Ok(event) => {
                    if let Err(error) = self.send(event).await {
                        self.status.store(false, Ordering::SeqCst);
                        error!("[BSC Listener] - {:?}", error);
                    }
                }
                Err(error) => {
                    // TODO add logging to db
                    error!("Error while decode event: {:?}", error);
                }
            }
        }
    }

    async fn send(&self, event: BscEventType) -> Result<(), Error> {
        self.db.add_extrinsic_bsc(&event).await?;
        self.tx.send(event).await.map_err(|_| Error::Send)?;

        Ok(())
    }
}
