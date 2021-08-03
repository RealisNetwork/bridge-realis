// use log::{error, info};
use realis_primitives::{Basic, TokenId};
use runtime::{realis_bridge::Call as RealisBridgeCall, AccountId, Call};
use sp_core::{sr25519, Pair, H160, H256 as Hash};
use sp_runtime::{generic, traits::BlakeTwo256};
use std::{fs, path::Path};
use substrate_api_client::{
    compose_extrinsic_offline, Api, BlockNumber, UncheckedExtrinsicV4, XtStatus,
};

type Header = generic::Header<BlockNumber, BlakeTwo256>;

fn from_path_to_account<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path).unwrap()
}

pub struct RealisSender {}

impl RealisSender {
    /// # Panics
    ///
    /// Connection to Realis.Network for transfers
    #[must_use]
    fn api() -> Api<sr25519::Pair> {
        // Get private key
        let pair = Pair::from_string(
            &*from_path_to_account("./../realis-sender/res/accounts.key"),
            None,
        )
        .unwrap();
        //
        let url = "rpc.realis.network";
        // Create substrate api with signer
        Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
            .map(|api| api.set_signer(pair))
            .unwrap()
    }

    /// # Panics
    ///
    /// Tranfer token from BSC to Realis.Network
    pub async fn send_token_to_realis(from: H160, to: AccountId, amount: u128) {
        let api = RealisSender::api();

        let head: Hash = api.get_finalized_head().unwrap().unwrap();
        let h: Header = api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
                from,
                to.clone(),
                amount * 10_000_000_000
            )),
            api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            api.genesis_hash,
            head,
            api.runtime_version.spec_version,
            api.runtime_version.transaction_version
        );

        // Send extrinsic transaction
        let tx_result = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => println!("Send extrinsic {:?}", hash),
            Err(error) => println!("Can`t send extrinsic {:?}", error),
        }
    }

    /// # Panics
    ///
    /// Tranfer nft from BSC to Realis.Network
    pub async fn send_nft_to_realis(
        from: H160,
        to: AccountId,
        token_id: TokenId,
        token_type: Basic,
    ) {
        let api = RealisSender::api();

        let head = api.get_finalized_head().unwrap().unwrap();
        let h: Header = api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_nft_to_realis(
                from,
                to.clone(),
                token_id,
                token_type
            )),
            api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            api.genesis_hash,
            head,
            api.runtime_version.spec_version,
            api.runtime_version.transaction_version
        );
        // Send extrinsic transaction
        let tx_result = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => println!("Send extrinsic {:?}", hash),
            Err(error) => println!("Can`t send extrinsic {:?}", error),
        }
    }

    /// # Panics
    ///
    /// Approve send from BSC to Realis.Network
    pub async fn send_token_approve_to_realis(from: AccountId, amount: u128) {
        let api = RealisSender::api();

        let head = api.get_finalized_head().unwrap().unwrap();
        let h: Header = api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_token_to_bsc_success(
                from.clone(),
                amount
            )),
            api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            api.genesis_hash,
            head,
            api.runtime_version.spec_version,
            api.runtime_version.transaction_version
        );
        // Send extrinsic transaction
        let tx_result = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => println!("Send extrinsic {:?}", hash),
            Err(error) => {
                println!("Can`t send extrinsic {:?}", error);
            }
        }
    }

    /// # Panics
    ///
    /// Approve send NFT from BSC to Realis.Network
    pub async fn send_nft_approve_to_realis_from_bsc(
        from: AccountId,
        token_id: TokenId,
    ) {
        let api = RealisSender::api();

        let head = api.get_finalized_head().unwrap().unwrap();
        let h: Header = api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_nft_to_bsc_success(
                from.clone(),
                token_id
            )),
            api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            api.genesis_hash,
            head,
            api.runtime_version.spec_version,
            api.runtime_version.transaction_version
        );
        // Send extrinsic transaction
        let tx_result = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => println!("Send extrinsic {:?}", hash),
            Err(error) => println!("Can`t send extrinsic {:?}", error),
        }
    }
}
