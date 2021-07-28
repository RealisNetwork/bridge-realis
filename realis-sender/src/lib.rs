use async_trait::async_trait;
use bsc_adapter::ContractEvents;
use logger::logger::{log, Type};
use primitive_types::U256;
use runtime::{realis_bridge::Call as RealisBridgeCall, Call, RealisBridge};
use sp_core::{sr25519, Pair};
use std::{fs, path::Path};
use substrate_api_client::{
    compose_extrinsic, compose_extrinsic_offline,
    sp_runtime::{traits::Header, AccountId32},
    Api, UncheckedExtrinsicV4, XtStatus,
};

fn from_path_to_account<P: AsRef<Path>>(path: P) -> String {
    let string = fs::read_to_string(path).unwrap();
    return string;
}

#[derive(Clone)]
pub struct RealisSender {
    api: Api<sr25519::Pair>,
}

impl RealisSender {
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
        // let from: AccountId32 =
        //     AccountId32::from_str("
        // 1aa0d5c594a4581ec17069ec9631cd6225d5fb403fe4d85c8ec8aa51833fdf7f")
        //         .unwrap();
        // Create extrinsic transaction
        let head = self.api.get_finalized_head().unwrap().unwrap();
        let h: Header = self.api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;

        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            self.api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_token_to_realis(
                to, *value
            )),
            self.api.get_nonce().unwrap(),
            Era::mortal(period, h.number.into()),
            self.api.genesis_hash,
            head,
            self.api.runtime_version.spec_version,
            self.api.runtime_version.transaction_version
        );

        println!("[+] Composed Extrinsic:\n {:?}\n", xt);
        // Send extrinsic transaction
        let tx_result =
            self.api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
        println!("{:?}", tx_result);
        match tx_result {
            Ok(hash) => {
                log(Type::Success, String::from("Send extrinsic"), &hash)
            }
            Err(error) => {
                log(Type::Error, String::from("Can`t send extrinsic"), &error)
            }
        }
    }

    async fn on_transfer_nft_to_realis<'a>(
        &self,
        to: AccountId,
        token_id: &U256,
        basic: u8,
    ) {
        // let from: AccountId32 =
        //     AccountId32::from_str("
        // 1aa0d5c594a4581ec17069ec9631cd6225d5fb403fe4d85c8ec8aa51833fdf7f")
        //         .unwrap();
        // Create extrinsic transaction
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            self.api.clone(),
            "RealisBridge",
            "transfer_nft_to_realis",
            GenericAddress::Id(to),
            token_id,
            basic
        );
        // Send extrinsic transaction
        let tx_result =
            self.api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        println!("{:?}", tx_result);

        // match tx_result {
        //     Ok(hash) => log(Type::Success, String::from("Send extrinsic"),
        // &hash),     Err(error) => log(Type::Error,
        // String::from("Can`t send extrinsic"), &error) }
    }
}
