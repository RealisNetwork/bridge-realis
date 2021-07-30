use log::{error, info};
use runtime::{realis_bridge::Call as RealisBridgeCall, Call};
use sp_core::{sr25519, Pair, H256 as Hash};
use sp_runtime::{generic, traits::BlakeTwo256};
use std::{fs, path::Path};
use substrate_api_client::{
    compose_extrinsic_offline, Api, BlockNumber, UncheckedExtrinsicV4, XtStatus,
};
use std::sync::mpsc::{Receiver, Sender};
use bridge_events::Events;

type Header = generic::Header<BlockNumber, BlakeTwo256>;

fn from_path_to_account<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path).unwrap()
}

pub struct RealisSender {
    // Get messages from realis adapter, bsc adapter, realis sender
    channel_from: Receiver<Events>,
    //
    // channel_to_bsc_sender
    api: Api<sr25519::Pair>,

}

impl RealisSender {
    /// # Panics
    ///
    /// Connection to Realis.Network for transfers
    #[must_use]
    pub fn new(channel_from: Receiver<Events>) -> Self {
        // Get private key
        let pair = Pair::from_string(
            &*from_path_to_account("./realis-sender/res/accounts.key"),
            None,
        )
        .unwrap();
        //
        let url = "rpc.realis.network";
        // Create substrate api with signer
        let api =
            Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
                .map(|api| api.set_signer(pair))
                .unwrap();

        RealisSender {
            channel_from,
            api
        }
    }

    pub async fn listen(&self) {
        loop {
            match self.channel_from.recv() {
                Ok(event) => {
                    match event {
                        Events::TokenSuccessOnBsc(from, amount) => {
                            let head = self.api.get_finalized_head().unwrap().unwrap();
                            let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
                            let period = 5;

                            #[allow(clippy::redundant_clone)]
                                let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                                self.api.clone().signer.unwrap(),
                                Call::RealisBridge(RealisBridgeCall::transfer_token_to_bsc_success(
                                    from.clone(),
                                    amount * 10_000_000_000
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
                                Ok(hash) => println!("Send extrinsic {:?}", hash),
                                Err(error) => println!("Can`t send extrinsic {:?}", error),
                            }
                        }
                        Events::NftSuccessOnBsc(from, token_id, _) => {
                            let head = self.api.get_finalized_head().unwrap().unwrap();
                            let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
                            let period = 5;

                            #[allow(clippy::redundant_clone)]
                                let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                                self.api.clone().signer.unwrap(),
                                Call::RealisBridge(RealisBridgeCall::transfer_nft_to_bsc_success(
                                    from.clone(),
                                    token_id
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
                                Ok(hash) => println!("Send extrinsic {:?}", hash),
                                Err(error) => println!("Can`t send extrinsic {:?}", error),
                            }
                        }
                        Events::TokenBscToRealis(address, amount) => {
                            let head: Hash = self.api.get_finalized_head().unwrap().unwrap();
                            let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
                            let period = 5;

                            #[allow(clippy::redundant_clone)]
                            let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                                self.api.clone().signer.unwrap(),
                                Call::RealisBridge(
                                    RealisBridgeCall::transfer_token_to_bsc_success(
                                        address.clone(),
                                        amount * 10_000_000_000
                                    )
                                ),
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
                                Ok(hash) => println!("Send extrinsic {:?}", hash),
                                Err(error) => println!("Can`t send extrinsic {:?}", error),
                            }
                        }
                        Events::NftBcsToRealis(address, token_id, token_type) => {
                            let head = self.api.get_finalized_head().unwrap().unwrap();
                            let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
                            let period = 5;

                            #[allow(clippy::redundant_clone)]
                            let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
                                self.api.clone().signer.unwrap(),
                                Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
                                    address.clone(),
                                    token_id,
                                    token_type
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
                                Ok(hash) => println!("Send extrinsic {:?}", hash),
                                Err(error) => println!("Can`t send extrinsic {:?}", error),
                            }
                        }

                        _ => {}
                    }
                }
                Err(_) => {}
            }
        }
    }
}