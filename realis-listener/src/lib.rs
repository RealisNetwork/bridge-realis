pub mod extrinsic_parser;

use db::Database;
use primitives::{block::Block, types::BlockNumber, Error};

use log::{error, info, warn};
use sp_core::sr25519;
use sp_runtime::{generic, traits::BlakeTwo256};
use substrate_api_client::{rpc::WsRpcClient, Api};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
};

use crate::extrinsic_parser::ExtrinsicParser;
use primitives::events::realis::RealisEventType;
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
    pub async fn new(url: &str, tx: Sender<RealisEventType>, status: Arc<AtomicBool>, db: Arc<Database>) -> Self {
        let (_, rx) = BlockListener::subscribe(url, Arc::clone(&status));
        Self { rx, tx, status, db }
    }

    /// # Errors
    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen_with_restore(&mut self, from: u64) {
        warn!("Start restore Realis!!!");
        let block_number = self.rx.recv().await.unwrap();
        for number in from..block_number {
            match BlockListener::get_block_sidecar(number).await {
                Ok(block) => {
                    match &self.db.update_block_realis(number).await {
                        Ok(_) => {
                            info!("Success add realis block to database");
                        }
                        Err(error) => {
                            error!("Can't add realis block to database with error: {:?}", error);
                        }
                    }
                    match self.process_block(block).await {
                        Ok(_) => {
                            info!("Block {} restored!", number);
                        }
                        Err(Error::Disconnected) => self.status.store(false, Ordering::SeqCst),
                        Err(Error::Send) => self.status.store(false, Ordering::SeqCst),
                        Err(error) => {
                            error!("Unable to restore block with error: {:?}", error);
                        }
                    }
                }
                Err(error) => error!("Unable to restore block: {:?}", error),
            }
        }

        self.listen().await;
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
                        let db = Arc::clone(&self.db);
                        match BlockListener::get_block_sidecar(block_number).await {
                            Ok(block) => {
                                 match &db.update_block_realis(block_number).await{
                                    Ok(_) => { info!("Success add realis block to database"); }
                                    Err(error) => {
                                        error!("Can't add realis block to database with error: {:?}", error);
                                    }
                                }
                                match self.process_block(block).await {
                                Ok(_) => info!("Block {} processed!", block_number),
                                Err(Error::Disconnected | Error::Send) =>
                                    self.status.store(false, Ordering::SeqCst),
                                Err(error) => {
                                    error!(
                                        "Unable to process block with error: {:?}",
                                        error
                                    );
                                }
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

    async fn get_block_sidecar(block_number: BlockNumber) -> Result<Block, Error> {
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
            .filter_map(|xt| ExtrinsicParser::new(xt.clone(), block_number))
            .map(extrinsic_parser::ExtrinsicParser::parse)
        {
            for event in events {
                warn!("send to BSC {:?}", event);
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
