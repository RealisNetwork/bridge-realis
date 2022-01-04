mod errors;
pub mod listener_builder;

use db::Database;
use errors::RpcError;
use frame_system::{EventRecord, Phase};
use log::{error, info, warn};
use rust_lib::healthchecker::HealthChecker;
use std::str::FromStr;
use tokio::select;
use web3::types::H160;

use primitives::events::realis::{RealisEventType, TransferNftToBsc, TransferTokenToBsc};
use runtime::{Block, Event};
use std::sync::Arc;
use substrate_api_client::{
    rpc::WsRpcClient,
    sp_runtime::app_crypto::{sp_core::H256, sr25519},
    Api, Hash,
};
use tokio::sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender};

pub struct BlockListener {
    rx: UnboundedReceiver<Hash>,
    tx: Sender<RealisEventType>,
    api: Api<sr25519::Pair, WsRpcClient>,
    health_checker: HealthChecker,
    db: Arc<Database>,
}

impl BlockListener {
    /// # Errors
    #[must_use]
    pub fn new(
        rx: UnboundedReceiver<Hash>,
        tx: Sender<RealisEventType>,
        api: Api<sr25519::Pair, WsRpcClient>,
        health_checker: HealthChecker,
        db: Arc<Database>,
    ) -> Self {
        Self {
            rx,
            tx,
            api,
            health_checker,
            db,
        }
    }

    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen(&mut self) {
        loop {
            let health_checker = self.health_checker.clone();
            select! {
                () = health_checker.is_alive() => break,
                result = self.execute() => {
                    match result {
                        Ok(block_number) => match &self.db.update_block_realis(block_number.into()).await {
                            Ok(_) => info!("Success add realis block to database"),
                            Err(error) => {
                                error!("Can't add realis block to database with error: {:?}", error);
                                self.health_checker.make_sick();
                            },
                        },
                        Err(error) => error!("{:?}", error),
                    }
                }
            }
        }
    }

    /// # Errors
    /// # Panics
    #[allow(clippy::match_same_arms)]
    pub async fn listen_with_restore(&mut self, from: u64, tx: UnboundedSender<H256>) {
        warn!("Start restore Realis!!!");
        let hash = self.rx.recv().await;

        match self.get_block(hash) {
            Ok(block) => {
                let block_number = u64::from(block.header.number);
                for number in from..block_number {
                    match self.api.get_storage_map("System", "BlockHash", number, None) {
                        Ok(Some(hash)) => {
                            info!("[Restore] - add to the queue - [{:^8}]", number);
                            let _result = tx.send(hash);
                        }
                        Ok(None) => warn!("[Restore] - missing block - [{:^8}]", number),
                        Err(error) => error!("[Restore] - {:?}]", error),
                    }
                    match self.execute().await {
                        Ok(block_number) => match &self.db.update_block_realis(block_number.into()).await {
                            Ok(_) => info!("Success add realis block to database"),
                            Err(error) => {
                                error!("Can't add realis block to database with error: {:?}", error);
                                self.health_checker.make_sick();
                            }
                        },
                        Err(error) => error!("{:?}", error),
                    }
                }

                self.listen().await;
            }
            Err(error) => {
                error!("Can't get block from hash: {:?}", error);
            }
        };
    }

    async fn execute(&mut self) -> Result<u32, RpcError> {
        let hash = self.rx.recv().await;
        let block = self.get_block(hash)?;
        let block_number = block.header.number;
        let events = self.get_events(hash)?;

        for event in events {
            if let Phase::ApplyExtrinsic(_) = event.phase {
                match event.event {
                    Event::RealisBridge(realis_bridge::Event::SendTokensToBsc(from, to, value, _)) => {
                        match H160::from_str(&format!("{:?}", to)) {
                            Ok(to) => {
                                match self
                                    .tx
                                    .send(RealisEventType::TransferTokenToBsc(TransferTokenToBsc {
                                        block: u64::from(block_number),
                                        hash: hash.unwrap(),
                                        from,
                                        to,
                                        amount: value,
                                    }))
                                    .await
                                {
                                    Ok(()) => info!("Success send to Binance Handler!"),
                                    Err(error) => {
                                        error!("Error transfer to Binance Handler {:?}", error);
                                        self.health_checker.make_sick();
                                    }
                                }
                            }
                            Err(error) => error!("Cannot parse account: {:?}", error),
                        }
                    }
                    Event::RealisBridge(realis_bridge::Event::TransferNftToBSC(from, to, token_id)) => {
                        match H160::from_str(&format!("{:?}", to)) {
                            Ok(dest) => {
                                match self
                                    .tx
                                    .send(RealisEventType::TransferNftToBsc(TransferNftToBsc {
                                        block: u64::from(block_number),
                                        hash: hash.unwrap(),
                                        from,
                                        dest,
                                        token_id,
                                    }))
                                    .await
                                {
                                    Ok(()) => info!("Success send to Binance Handler!"),
                                    Err(error) => {
                                        error!("Error transfer to Binance Handler {:?}", error);
                                        self.health_checker.make_sick();
                                    }
                                }
                            }
                            Err(error) => error!("Cannot parse account: {:?}", error),
                        }
                    }
                    event => warn!("[Event] - skipping - {:?}", event),
                }
            }
        }

        Ok(block_number)
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
}
