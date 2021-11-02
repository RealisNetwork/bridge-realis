use log::{error, info};
use primitives::{events::BscEventType, Error};
use realis_bridge::Call as RealisBridgeCall;
use runtime::Call;
use rust_lib::healthchecker::HealthChecker;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use db::Database;
use primitives::db::Status;
use substrate_api_client::{
    compose_extrinsic_offline,
    rpc::WsRpcClient,
    sp_runtime::app_crypto::{sp_core::H160, sr25519},
    Api, Pair, UncheckedExtrinsicV4, XtStatus,
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

        match self
            .db
            .update_status_bsc(&request.get_hash().to_string(), Status::InProgress)
            .await
        {
            Ok(_) => info!("Success update realis status InProgress"),
            Err(error) => error!("Error while updating realis status: {:?}", error),
        };

        let tx_result = match request {
            BscEventType::TransferTokenToRealis(request, ..) => {
                let account_id = request.to.clone();
                let amount = request.amount;
                let bsc_account = H160::from_slice(request.from.as_ref());

                let call: Call = Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
                    bsc_account,
                    account_id,
                    amount,
                ));

                let tx: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                    self.api.signer.clone().unwrap(),
                    call,
                    self.api.get_nonce().unwrap(),
                    Era::Immortal,
                    self.api.genesis_hash,
                    self.api.genesis_hash,
                    self.api.runtime_version.spec_version,
                    self.api.runtime_version.transaction_version
                );

                self.api.send_extrinsic(tx.hex_encode(), XtStatus::InBlock)
            }
            BscEventType::TransferNftToRealis(request, ..) => {
                let account_id = request.dest.clone();
                let token_id = request.token_id;
                let bsc_account = H160::from_slice(request.from.as_ref());

                let call: Call = Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
                    bsc_account,
                    account_id,
                    token_id,
                ));

                let tx: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                    self.api.signer.clone().unwrap(),
                    call,
                    self.api.get_nonce().unwrap(),
                    Era::Immortal,
                    self.api.genesis_hash,
                    self.api.genesis_hash,
                    self.api.runtime_version.spec_version,
                    self.api.runtime_version.transaction_version
                );

                self.api.send_extrinsic(tx.hex_encode(), XtStatus::InBlock)
            }
        };

        let status = match tx_result {
            Ok(result) => {
                info!("Hash: {:?}", result);
                Status::Success
            }
            Err(error) => {
                error!("Cannot send extrinsic: {:?}", error);
                Status::Error
            }
        };

        match self.db.update_status_bsc(&request.get_hash().to_string(), status).await {
            Ok(_) => info!("Success update realis status"),
            Err(error) => error!("Error while updating realis status: {:?}", error),
        }

        Ok(())
    }
}
