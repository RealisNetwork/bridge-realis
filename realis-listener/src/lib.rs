pub mod extrinsic_parser;
mod errors;
mod listener_builder;

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

use rust_lib::blockchain::block::Event;
use tokio::time::{sleep, Duration};

pub struct BlockListener {
    rx: UnboundedReceiver<Hash>,
    tx: Sender<(Value, String)>,
    api: Api<sr25519::Pair, WsRpcClient>,
    status: Arc<AtomicBool>,
    db: Database,
}

// TODO refactor
impl BlockListener {
    /// # Errors
    pub async fn new(
        url: &str,
        tx: Sender<(Value, String)>,
        api: Api<sr25519::Pair, WsRpcClient>,
        status: Arc<AtomicBool>,
        db: Database
    ) -> Self {
        let (_, rx) = BlockListener::subscribe(url, Arc::clone(&status));
        Self {
            rx,
            tx,
            api,
            status,
            db
        }
    }

    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen(&mut self) {
        loop {
            select! {
                () = HealthChecker::is_alive(Arc::clone(&self.status)) => break,
                result = self.execute() => {
                    match result {
                        Ok(block_number) => match self.db.update_block_realis(block_number.into()).await {
                            Ok(_) => info!("Success add realis block to database"),
                            Err(error) => {
                                error!("Can't add realis block to database with error: {:?}", error);
                                self.status.store(false, Ordering::SeqCst);
                            },
                        },
                        Err(error) => error!("{:?}", error),
                    }
                }
            }
        }
    }

    // TODO find transaction type and get fields(extrinsic_parser::parse_args())
    async fn execute(&mut self) -> Result<u32, RpcError> {
        let hash = self.rx.recv().await;
        let block = self.get_block(hash)?;
        let block_number = block.number;
        let events = self.get_events(hash)?;

        for event in events {
            if let Phase::ApplyExtrinsic(index) = event.phase {
                match event.event {
                    // TODO change to bridge events
                    Event::RealisBridge(pallet_bridge::Event::BatchCompleted) => {
                        match self.process_block(block.clone()).await {
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
                    }
                    // TODO change to bridge events
                    Event::Utility(pallet_utility::Event::BatchInterrupted(_, _)) => {
                        match self.process_block(block.clone()).await {
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
                    }
                    event => warn!("[Event] - skipping - {:?}", event),
                }
            }
        }

        Ok(block_number as u32)
    }

    fn get_block(&self, hash: Option<H256>) -> Result<Block, RpcError> {
        self.api
            .get_block(hash)
            .map_err(|_| RpcError::Api)?
            .ok_or(RpcError::BlockNotFound)
    }

    fn get_events(&self, hash: Option<H256>) -> Result<Vec<EventRecord<Event, H256>>, RpcError> {
        self.api
            .get_storage_value::<Vec<EventRecord<Event, H256>>>("System", "Events", hash)
            .map_err(|_| RpcError::Api)?
            .ok_or(RpcError::EventsNotFound)
    }

    // TODO get extrinsics
    async fn process_block(&self, block: Block) -> Result<(), Error> {
        let block_number = block.number;
        for events in block
            .extrinsics
            .iter()
            .filter_map(|xt| ExtrinsicParser::new(xt.clone(), block_number.into()))
            .map(extrinsic_parser::ExtrinsicParser::parse())
        {
            for event in events {
                warn!("send to BSC {:?}", event);
                match self.db.add_extrinsic_realis(&event).await {
                    Ok(()) => info!("Success add to Database!"),
                    Err(error) => error!("Cannot add extrinsic {:?}", error),
                };
                self.tx.send(hash).await.map_err(|_| Error::Send)?;
            }
        }

        Ok(())
    }

    // TODO remove
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

    // /// # Errors
    // /// # Panics
    // #[allow(clippy::match_same_arms)]
    // pub async fn listen_with_restore(&mut self, from: u64) {
    //     warn!("Start restore Realis!!!");
    //     let block_number = self.rx.recv().await.unwrap();
    //     for number in from..block_number {
    //         match BlockListener::get_block_sidecar(number).await {
    //             Ok(block) => {
    //                 match &self.db.update_block_realis(number).await {
    //                     Ok(_) => {
    //                         info!("Success add realis block to database");
    //                     }
    //                     Err(error) => {
    //                         error!("Can't add realis block to database with error: {:?}", error);
    //                     }
    //                 }
    //                 match self.process_block(block).await {
    //                     Ok(_) => {
    //                         info!("Block {} restored!", number);
    //                     }
    //                     Err(Error::Disconnected) => self.status.store(false, Ordering::SeqCst),
    //                     Err(Error::Send) => self.status.store(false, Ordering::SeqCst),
    //                     Err(error) => {
    //                         error!("Unable to restore block with error: {:?}", error);
    //                     }
    //                 }
    //             }
    //             Err(error) => error!("Unable to restore block: {:?}", error),
    //         }
    //     }
    //
    //     self.listen().await;
    // }
}
