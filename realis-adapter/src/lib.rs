use log::{error, info};
use primitives::{events::BscEventType, Error};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use substrate_api_client::{
    compose_extrinsic, rpc::WsRpcClient, sp_runtime::app_crypto::sr25519, Api, Pair, UncheckedExtrinsicV4,
    XtStatus,
};
use tokio::sync::mpsc::Receiver;

pub struct RealisAdapter {
    rx: Receiver<BscEventType>,
    status: Arc<AtomicBool>,
    api: Api<sr25519::Pair, WsRpcClient>,
}

impl RealisAdapter {
    #[must_use]
    /// # Panics
    pub fn new(rx: Receiver<BscEventType>, status: Arc<AtomicBool>, url: &str, master_key: sr25519::Pair) -> Self {
        let client = WsRpcClient::new(url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client)
            .unwrap()
            .set_signer(master_key);
        Self { rx, status, api }
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
                let bsc_account = primitive_types::H160::from_slice(request.from.as_ref());
                let tx: UncheckedExtrinsicV4<_> = compose_extrinsic!(
                    self.api,
                    "RealisBridge",
                    "transfer_token_to_realis",
                    bsc_account,
                    account_id,
                    amount
                );

                let tx_result = self.api.send_extrinsic(tx.hex_encode(), XtStatus::InBlock);
                match tx_result {
                    Ok(result) => info!("Hash: {:?}", result),
                    Err(error) => error!("Cannot send extrinsic: {:?}", error),
                }
                Ok(())
            }
            BscEventType::TransferNftToRealisSuccess(request, ..) => {
                let account_id = request.dest.clone();
                let token_id = request.token_id;
                let bsc_account = primitive_types::H160::from_slice(request.from.as_ref());

                let tx: UncheckedExtrinsicV4<_> = compose_extrinsic!(
                    self.api,
                    "RealisBridge",
                    "transfer_nft_to_realis",
                    bsc_account,
                    account_id,
                    token_id
                );

                let tx_result = self.api.send_extrinsic(tx.hex_encode(), XtStatus::InBlock);
                match tx_result {
                    Ok(result) => info!("Hash: {:?}", result),
                    Err(error) => error!("Cannot send extrinsic: {:?}", error),
                }
                Ok(())
            }
            BscEventType::TransferTokenToRealisError(..) | BscEventType::TransferNftToRealisError(..) => Ok(()),
        }
    }
}
