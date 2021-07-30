use async_trait::async_trait;
use log::{error, info};
use realis_adapter::BridgeEvents;
use runtime::realis_bridge::TokenId;
use secp256k1::SecretKey;
use sp_core::H160;
use std::{fs, path::Path, str::FromStr};
use utils::contract;
use web3::types::{Address, U256};

pub struct BscSender {
    // web3: web3::Web3<WebSocket>,
    // contract: Contract<WebSocket>,
    wallet_key: SecretKey,
}

impl BscSender {
    pub async fn new() -> BscSender {
        let wallet_key =
            BscSender::read_file_for_secret_key("bsc-sender/res/accounts.key");

        BscSender {
            // web3,
            // contract,
            wallet_key,
        }
    }

    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}

#[async_trait]
impl BridgeEvents for BscSender {
    async fn on_transfer_token_to_bsc<'a>(&self, to: H160, value: u128) {
        let contract = contract::token_new().await;
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(value) * 100_000_000;

        let result = contract
            .signed_call_with_confirmations(
                "transfer",
                (to, value),
                web3::contract::Options::default(),
                1,
                &self.wallet_key,
            )
            .await;

        match result {
            Ok(value) => info!("Transaction success {:?}", value),
            Err(err) => error!("Transaction fail {:?}", err),
        }
    }

    async fn on_transfer_nft_to_bsc<'a>(
        &self,
        to: H160,
        token_id: TokenId,
        _token_type: u8,
    ) {
        let contract = contract::nft_new().await;

        let to: Address = Address::from(to.0);

        let result = contract
            .signed_call_with_confirmations(
                "safeMint",
                (to, token_id),
                web3::contract::Options::default(),
                1,
                &self.wallet_key,
            )
            .await;

        match result {
            Ok(value) => info!("Transaction success {:?}", value),
            Err(err) => error!("Transaction fail {:?}", err),
        }
    }
}
