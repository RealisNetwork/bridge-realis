use log::{error, info};
use primitives::Error;

use rust_lib::healthchecker::HealthChecker;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use db::Database;
use primitives::{db::Status, events::bsc::BscEventType};
use substrate_api_client::{
    compose_extrinsic_offline, rpc::WsRpcClient, sp_runtime::app_crypto::sr25519, Api, Pair, UncheckedExtrinsicV4,
    XtStatus,
};
use tokio::{select, sync::mpsc::Receiver};

pub struct RealisAdapter {
    rx: Receiver<BscEventType>,
    status: Arc<AtomicBool>,
    api: Api<sr25519::Pair, WsRpcClient>,
    db: Arc<Database>,
}

impl RealisAdapter {
    #[must_use]
    /// # Panics
    pub fn new(
        rx: Receiver<BscEventType>,
        status: Arc<AtomicBool>,
        url: &str,
        master_key: sr25519::Pair,
        db: Arc<Database>,
    ) -> Self {
        let client = WsRpcClient::new(url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client)
            .unwrap()
            .set_signer(master_key);
        Self { rx, status, api, db }
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
                                error!("Cannot send transaction {:?}", error);
                                self.status.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute(&mut self, request: &BscEventType) -> Result<(), Error> {
        info!("Start send transaction");

        match self.db.update_status_bsc(&request.get_hash(), Status::InProgress).await {
            Ok(_) => info!("Success update realis status InProgress"),
            Err(error) => error!("Error while updating realis status: {:?}", error),
        };

        let tx: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            self.api.signer.clone().unwrap(),
            request.get_call(),
            self.api.get_nonce().unwrap(),
            Era::Immortal,
            self.api.genesis_hash,
            self.api.genesis_hash,
            self.api.runtime_version.spec_version,
            self.api.runtime_version.transaction_version
        );

        let tx_result = self
            .api
            .send_extrinsic(tx.hex_encode(), XtStatus::InBlock)
            .map_err(Error::Api);

        let status = match tx_result {
            Ok(_) => Status::Success,
            Err(_) => Status::Error,
        };

        match self.db.update_status_bsc(&request.get_hash().to_string(), status).await {
            Ok(_) => info!("Success update realis status"),
            Err(error) => error!("Error while updating realis status: {:?}", error),
        }

        tx_result.map(|_| ())
    }
}
