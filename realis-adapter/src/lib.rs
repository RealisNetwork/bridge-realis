use db::Database;
use primitives::{
    db::Status,
    events::{bsc::BscEventType, realis::RealisEventType, traits::Event},
    Error,
};

use frame_system::{EventRecord, Phase};
use runtime::{Address, Block, Event as RuntimeEvent};
use rust_lib::healthchecker::HealthChecker;
use substrate_api_client::{
    compose_extrinsic_offline,
    rpc::WsRpcClient,
    sp_runtime::app_crypto::{sp_core::H256, sr25519},
    Api, Hash, Pair, XtStatus,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use log::{error, info};
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
};

pub struct RealisAdapter {
    rx: Receiver<BscEventType>,
    tx: Sender<RealisEventType>,
    status: Arc<AtomicBool>,
    api: Api<sr25519::Pair, WsRpcClient>,
    db: Arc<Database>,
}

impl RealisAdapter {
    #[must_use]
    /// # Panics
    pub fn new(
        rx: Receiver<BscEventType>,
        tx: Sender<RealisEventType>,
        status: Arc<AtomicBool>,
        url: &str,
        master_key: sr25519::Pair,
        db: Arc<Database>,
    ) -> Self {
        let client = WsRpcClient::new(url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client)
            .unwrap()
            .set_signer(master_key);
        Self {
            rx,
            tx,
            status,
            api,
            db,
        }
    }

    /// # Panics
    /// # Errors
    pub async fn handle(mut self) {
        loop {
            select! {
                () = HealthChecker::is_alive(Arc::clone(&self.status)) => break,
                option = self.rx.recv() => {
                    if let Some(message) = option {
                        match self.execute(&message).await {
                            Ok(_) => {
                                info!("Success send transaction to BSC!");
                            }
                            Err(error) => {
                                let rollback_request = match message {
                                    BscEventType::TransferNftToRealis(request, ..) => {
                                        Some(RealisEventType::TransferNftToRealisFail(request))
                                    }
                                    BscEventType::TransferTokenToRealis(request, ..) => {
                                        Some(RealisEventType::TransferTokenToRealisFail(request))
                                    }
                                    // If rollback request fail
                                    _ => None
                                };
                                if let Some(rollback_request) = rollback_request {
                                    // TODO handle result
                                    let _result = self.tx.send(rollback_request).await;
                                } else {
                                    error!("Rollback fail: {:?}", error);
                                    self.status.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute(&self, request: &BscEventType) -> Result<(), Error> {
        match request {
            BscEventType::TransferTokenToRealis(event) => self.process(event).await,
            BscEventType::TransferNftToRealis(event) => self.process(event).await,
            BscEventType::TransferTokenToBscFail(event) => self.rollback(event).await,
            BscEventType::TransferNftToBscFail(event) => self.rollback(event).await,
        }
    }

    async fn process(&self, event: &impl Event) -> Result<(), Error> {
        // TODO handle result
        let _result = self.db.update_status_bsc(&event.get_hash(), Status::InProgress).await;
        let tx_result = self.send_to_blockchain(event);
        // TODO handle result
        let _result = self
            .db
            .update_status_bsc(
                &event.get_hash(),
                tx_result.as_ref().map(|_| Status::Success).unwrap_or(Status::Error),
            )
            .await;

        tx_result
    }

    async fn rollback(&self, event: &impl Event) -> Result<(), Error> {
        let tx_result = self.send_to_blockchain(event);
        // TODO handle result
        let _result = self
            .db
            .update_status_realis(
                &event.get_hash(),
                tx_result.as_ref().map(|_| Status::Rollbacked).unwrap_or(Status::Error),
            )
            .await;
        tx_result
    }

    fn send_to_blockchain(&self, event: &impl Event) -> Result<(), Error> {
        let tx = compose_extrinsic_offline!(
            self.api.signer.clone().unwrap(),
            event.get_realis_call(),
            self.api.get_nonce().unwrap(),
            Era::Immortal,
            self.api.genesis_hash,
            self.api.genesis_hash,
            self.api.runtime_version.spec_version,
            self.api.runtime_version.transaction_version
        );

        let hash = self
            .api
            .send_extrinsic(tx.hex_encode(), XtStatus::Finalized)
            .map_err(Error::Api)?;

        self.check_extrinsic(hash)
    }

    fn check_extrinsic(&self, block_hash: Option<Hash>) -> Result<(), Error> {
        let block = self
            .api
            .get_block::<Block>(block_hash)
            .map_err(Error::Api)?
            .ok_or(Error::Custom(String::from("Missing block!")))?;

        let events = self
            .api
            .get_storage_value::<Vec<EventRecord<RuntimeEvent, H256>>>("System", "Events", block_hash)
            .map_err(Error::Api)?
            .ok_or(Error::Custom(String::from("Missing events!")))?;

        for event in events {
            if let RuntimeEvent::System(frame_system::Event::ExtrinsicSuccess(_)) = event.event {
                if let Phase::ApplyExtrinsic(index) = event.phase {
                    let xt = block.extrinsics.get(index as usize).unwrap();
                    if xt.signature.is_some() {
                        if let Address::Id(account_id) = &xt.signature.as_ref().unwrap().0 {
                            if account_id.clone() == self.api.signer_account().unwrap() {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        Err(Error::Custom(String::from("Not confirmation found")))
    }
}
