mod tx_sender;

use crate::tx_sender::TxSender;

use db::Database;
use log::{error, info, warn};
use primitives::events::bsc::{BscEventType};

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;
use web3::{
    self,
    futures::StreamExt,
    transports::WebSocket,
    types::{Address, BlockNumber, Transaction},
    Web3,
};

pub struct BlockListener {
    web3: Web3<WebSocket>,
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
    db: Arc<Database>,
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
        token_contract: &str,
        nft_contract: &str,
    ) -> Self {
        // TODO get rid of unwrap
        let ws = web3::transports::WebSocket::new(&url).await.unwrap();
        let web3 = web3::Web3::new(ws);
        // TODO get rid of unwraps
        let token_contract = Address::from_str(token_contract).unwrap();
        let nft_contract = Address::from_str(nft_contract).unwrap();

        Self {
            web3,
            tx,
            status,
            db,
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
            let tx_sender = TxSender::new(self.tx.clone(), self.status.clone()).await;

            if account == self.token_contract {
                tx_sender.send_tokens(transaction, self.web3.clone(), &self.db).await;
            } else if account == self.nft_contract {
                tx_sender.send_nft(transaction, self.web3.clone(), &self.db).await;
            }
        }
    }
}
