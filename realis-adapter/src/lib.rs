use db::Database;
use primitives::{
    block::Block, types::BlockNumber, Error, events::EventType
};

use futures::Future;
use log::{error, info, warn};
use sp_core::sr25519;
use sp_runtime::{generic, traits::BlakeTwo256};
use substrate_api_client::{rpc::WsRpcClient, Api};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc,
};

pub struct BlockListener {
    rx: UnboundedReceiver<BlockNumber>,
    tx: Sender<Result<EventType, Error>>,
    status: Arc<AtomicBool>,
}

impl BlockListener {
    /// # Errors
    pub fn new(
        url: &str,
        tx: Sender<Result<EventType, Error>>,
        status: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        let (_, rx) = BlockListener::subscribe(url, Arc::clone(&status));
        Ok(Self { rx, tx, status })
    }

    /// # Errors
    pub async fn new_with_restore(
        url: &str,
        tx: Sender<Result<EventType, Error>>,
        db: Database,
        status: Arc<AtomicBool>,
    ) -> Result<(Self, impl Future), Error> {
        let (tx_copy, mut rx) = BlockListener::subscribe(url, Arc::clone(&status));
        let current = rx.recv().await.ok_or(Error::Disconnected)?;
        let last = db.get_last_block().await?;
        Ok((
            Self { rx, tx, status },
            BlockListener::restore(tx_copy, current, last),
        ))
    }

    /// # Panics
    pub async fn restore(
        tx: UnboundedSender<BlockNumber>,
        current: BlockNumber,
        last_processed: BlockNumber,
    ) {
        for block_number in last_processed..current {
            if let Err(error) = tx.send(block_number) {
                panic!("In restore: {:?}", error);
            }
        }
    }

    /// # Panics
    pub async fn listen(&mut self) {
        loop {
            select! {
                _ = is_alive(Arc::clone(&self.status)) => break,
                option = self.rx.recv() => {
                    if let Some(block_number) = option {
                        match BlockListener::get_block(block_number).await {
                            Ok(block) => match self.process_block(block).await {
                                Ok(_) => info!("Block {} processed!", block_number),
                                Err(Error::Disconnected) =>
                                    self.terminate(Error::Disconnected).await,
                                Err(Error::Send) =>
                                    self.terminate(Error::Send).await,
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
                    match sync_rx.recv().map(|header| {
                        serde_json::from_str::<generic::Header<BlockNumber, BlakeTwo256>>(
                            &header,
                        )
                    }) {
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
        let request = format!(
            "http://135.181.18.215:8080/blocks/{}",
            block_number.to_string()
        );
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

        for events in block
            .extrinsics
            .iter()
            .filter_map(|xt| BatchParser::new(xt.clone(), block_number))
            .map(|batch| batch.parse())
        {
            for event in events {
                warn!("{:?}", event);
                self.tx.send(Ok(event)).await.map_err(|_| Error::Send)?;
            }
        }

        Ok(())
    }

    #[allow(unused_must_use)]
    async fn terminate(&mut self, error: Error) {
        warn!("Terminate listener with error: {:?}", error);
        self.rx.close();
        self.tx.send(Err(error)).await;

        self.status.store(false, Ordering::SeqCst);
    }
}