use crate::BlockListener;
use db::Database;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc,
};

use sp_runtime::{generic, traits::BlakeTwo256};
use substrate_api_client::{
    rpc::WsRpcClient,
    sp_runtime::app_crypto::{sp_core::H256, sr25519},
    Api, BlockNumber,
};

use log::error;
use serde_json::Value;
use tokio::sync::mpsc::{unbounded_channel, Sender, UnboundedSender};

pub struct BlockListenerBuilder {
    url: String,
    tx: Sender<(Value, String)>,
    status: Arc<AtomicBool>,
    db: Database,
}

impl BlockListenerBuilder {
    pub fn new(
        url: &str,
        tx: Sender<(Value, String)>,
        status: Arc<AtomicBool>,
        db: Database,
    ) -> Self {
        Self {
            url: String::from(url),
            tx,
            status,
            db,
        }
    }

    pub fn build(self) -> (BlockListener, UnboundedSender<H256>) {
        let client = WsRpcClient::new(&self.url);
        let api = Api::<sr25519::Pair, WsRpcClient>::new(client).unwrap();
        let (async_tx, async_rx) = unbounded_channel();

        // try use tokio spawn
        std::thread::spawn({
            let async_tx = async_tx.clone();
            let api = api.clone();
            let status = Arc::clone(&self.status);

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
                        serde_json::from_str::<generic::Header<BlockNumber, BlakeTwo256>>(&header)
                    }) {
                        Ok(Ok(header)) => {
                            if let Err(error) = async_tx.send(header.hash()) {
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

        (
            BlockListener::new(async_rx, self.tx, self.status, api, self.db),
            async_tx,
        )
    }
}