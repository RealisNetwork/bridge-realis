pub mod block_parser;

use db::Database;
use primitives::{block::Block, events::RealisEventType, types::BlockNumber, Error};

use futures::Future;
use log::{error, info, warn};
use sp_core::sr25519;
use sp_runtime::{generic, traits::BlakeTwo256};
use substrate_api_client::{rpc::WsRpcClient, Api};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
};

use crate::block_parser::BlockParser;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc,
};
use tokio::time::{sleep, Duration};

pub struct BlockListener {
    rx: UnboundedReceiver<BlockNumber>,
    tx: Sender<RealisEventType>,
    status: Arc<AtomicBool>,
    db: Arc<Database>,
}

impl BlockListener {
    /// # Errors
    pub fn new(
        url: &str,
        tx: Sender<RealisEventType>,
        status: Arc<AtomicBool>,
        db: Arc<Database>,
    ) -> Result<Self, Error> {
        let (_, rx) = BlockListener::subscribe(url, Arc::clone(&status));
        Ok(Self { rx, tx, status, db })
    }

    /// # Errors
    pub async fn new_with_restore(
        url: &str,
        tx: Sender<RealisEventType>,
        db: Arc<Database>,
        status: Arc<AtomicBool>,
    ) -> Result<(Self, impl Future), Error> {
        let (tx_copy, mut rx) = BlockListener::subscribe(url, Arc::clone(&status));
        let current = rx.recv().await.ok_or(Error::Disconnected)?;
        let last = db.get_last_block_realis().await?;
        Ok((
            Self { rx, tx, status, db },
            BlockListener::restore(tx_copy, current, last),
        ))
    }

    /// # Panics
    pub async fn restore(tx: UnboundedSender<BlockNumber>, current: BlockNumber, last_processed: BlockNumber) {
        for block_number in last_processed..current {
            if let Err(error) = tx.send(block_number) {
                panic!("In restore: {:?}", error);
            }
        }
    }

    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen(&mut self) {
        loop {
            select! {
                _ = is_alive(Arc::clone(&self.status)) => break,
                option = self.rx.recv() => {
                    if let Some(block_number) = option {
                        info!("Start process block!");
                        match BlockListener::get_block(block_number).await {
                            // TODO update block in table?
                            Ok(block) => match self.process_block(block).await {
                                Ok(_) => info!("Block {} processed!", block_number),
                                Err(Error::Disconnected) => self.status.store(false, Ordering::SeqCst),
                                Err(Error::Send) => self.status.store(false, Ordering::SeqCst),
                                Err(error) => {
                                    error!(
                                        "Unable to process block with error: {:?}",
                                        error
                                    );
                                }
                            },
                            Err(error) => error!("Unable to get block: {:?}", error),
                        }
                    }
                }
            }
        }
    }

    fn subscribe(
        url: &str,
        status: Arc<AtomicBool>,
    ) -> (UnboundedSender<BlockNumber>, UnboundedReceiver<BlockNumber>) {
        let client = WsRpcClient::new(url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client).unwrap();
        let (async_tx, async_rx) = unbounded_channel();

        std::thread::spawn({
            let async_tx = async_tx.clone();

            move || {
                let (sync_tx, sync_rx) = channel();

                if let Err(_error) = api.subscribe_finalized_heads(sync_tx) {
                    status.store(false, Ordering::SeqCst);
                    return;
                }

                loop {
                    if !status.load(Ordering::Acquire) {
                        break;
                    }
                    match sync_rx
                        .recv()
                        .map(|header| serde_json::from_str::<generic::Header<BlockNumber, BlakeTwo256>>(&header))
                    {
                        Ok(Ok(header)) => {
                            if let Err(error) = async_tx.send(header.number) {
                                error!("{:?}", error);
                                return;
                            }
                        }
                        Ok(Err(error)) => {
                            error!("{:?}", error);
                        }
                        Err(error) => {
                            error!("Terminating with error: {:?}", error);
                            status.store(false, Ordering::SeqCst);
                        }
                    }
                }
            }
        });

        (async_tx, async_rx)
    }

    async fn get_block(block_number: BlockNumber) -> Result<Block, Error> {
        // Create request
        let request = format!("http://135.181.18.215:8080/blocks/{:?}", block_number);
        // Send request and wait response
        reqwest::get(request)
            .await
            .map_err(|_| Error::Disconnected)?
            .json()
            .await
            .map_err(|_| Error::CannotDecode)
    }

    async fn process_block(&self, block: Block) -> Result<(), Error> {
        let block_number = block.number;
        // self.db.update_block_realis(block_number).await;
        for events in block
            .extrinsics
            .iter()
            .filter_map(|xt| BlockParser::new(xt.clone(), block_number))
            .map(|block| block.parse())
        {
            for event in events {
                warn!("send to BSC {:?}", event);
                // TODO update status to got
                match self.db.add_extrinsic_realis(&event).await {
                    Ok(()) => info!("Success add to Database!"),
                    Err(error) => error!("Cannot add extrinsic {:?}", error),
                };
                self.tx.send(event).await.map_err(|_| Error::Send)?;
            }
        }

        Ok(())
    }
}

pub async fn is_alive(status: Arc<AtomicBool>) {
    while status.load(Ordering::Acquire) {
        sleep(Duration::from_millis(10000)).await;
    }
}
