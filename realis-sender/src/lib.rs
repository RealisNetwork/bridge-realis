use async_trait::async_trait;
use bsc_adapter::ContractEvents;
use primitive_types::U256;
use runtime::{realis_bridge::Call as RealisBridgeCall, AccountId, Call};
use sp_core::{sr25519, Pair, H256 as Hash};
use sp_runtime::{generic, traits::BlakeTwo256};
use std::{fs, path::Path};
use substrate_api_client::{
    compose_extrinsic_offline, Api, BlockNumber, UncheckedExtrinsicV4, XtStatus,
};

use slog::{error, info};
use utils::logger;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

fn from_path_to_account<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path).unwrap()
}

#[derive(Clone)]
pub struct RealisSender {
    api: Api<sr25519::Pair>,
}

impl RealisSender {
    /// # Panics
    ///
    /// Conection to Realis.Network for transfers
    #[must_use]
    pub fn new(url: &str) -> Self {
        // Get private key
        let pair = Pair::from_string(
            &*from_path_to_account("./realis-sender/res/accounts.key"),
            None,
        )
        .unwrap();
        // Create substrate api with signer
        let api =
            Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
                .map(|api| api.set_signer(pair))
                .unwrap();

        RealisSender { api }
    }
}

#[async_trait]
impl ContractEvents for RealisSender {
    async fn on_transfer_token_to_realis<'a>(
        &self,
        to: AccountId,
        value: &u128,
    ) {
        let log = logger::new();

        let head: Hash = self.api.get_finalized_head().unwrap().unwrap();
        let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            self.api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
                to.clone(),
                *value * 10_000_000_000
            )),
            self.api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            self.api.genesis_hash,
            head,
            self.api.runtime_version.spec_version,
            self.api.runtime_version.transaction_version
        );

        // Send extrinsic transaction
        let tx_result =
            self.api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => info!(log, "Send extrinsic {:?}", hash),
            Err(error) => error!(log, "Can`t send extrinsic {:?}", error),
        }
    }

    async fn on_transfer_nft_to_realis<'a>(
        &self,
        to: AccountId,
        token_id: &U256,
        basic: u8,
    ) {
        let log = logger::new();

        let head = self.api.get_finalized_head().unwrap().unwrap();
        let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            self.api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
                to.clone(),
                *token_id,
                basic
            )),
            self.api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            self.api.genesis_hash,
            head,
            self.api.runtime_version.spec_version,
            self.api.runtime_version.transaction_version
        );
        // Send extrinsic transaction
        let tx_result =
            self.api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => info!(log, "Send extrinsic {:?}", hash),
            Err(error) => error!(log, "Can`t send extrinsic {:?}", error),
        }
    }
}
