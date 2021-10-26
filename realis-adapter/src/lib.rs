use log::{error, info};
use primitives::{events::BscEventType, Error};
use runtime::AccountId;
use rust_lib::config::Config;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use substrate_api_client::{
    compose_extrinsic,
    rpc::WsRpcClient,
    sp_runtime::{
        app_crypto::{sp_core::Hasher, sr25519},
        traits::BlakeTwo256,
    },
    Api, Pair, UncheckedExtrinsicV4, XtStatus,
};
use tokio::sync::mpsc::Receiver;

pub struct RealisAdapter {
    rx: Receiver<BscEventType>,
    status: Arc<AtomicBool>,
    url: String,
    api: Api<sr25519::Pair, WsRpcClient>,
}

impl RealisAdapter {
    pub fn new(
        rx: Receiver<BscEventType>,
        status: Arc<AtomicBool>,
        url: String,
        master_key: sr25519::Pair,
    ) -> Self {
        let client = WsRpcClient::new(&url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client)
            .unwrap()
            .set_signer(master_key);
        Self { rx, status, url, api }
    }

    pub async fn handle(mut self) {
        // TODO check handle still_alive status
        while let Some(request) = self.rx.recv().await {
            match self.execute(&request).await {
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

    async fn execute(&mut self, request: &BscEventType) -> Result<(), Error> {
        info!("Start send transaction");

        match request {
            BscEventType::TransferTokenToRealisSuccess(request, ..) => {
                let account_id = request.to.clone();
                let amount = request.amount;
                let tx: UncheckedExtrinsicV4<_> =
                    compose_extrinsic!(self.api, "RealisBridge", "transfer_token_to_realis", account_id, amount);

                let tx_result = self.api.send_extrinsic(tx.hex_encode(), XtStatus::InBlock);
                Ok(())
            }
            BscEventType::TransferNftToRealisSuccess(request, ..) => Ok(()),
            BscEventType::TransferTokenToRealisError(..) | BscEventType::TransferNftToRealisError(..) => Ok(()),
        }
    }

    fn connect() {}
}
