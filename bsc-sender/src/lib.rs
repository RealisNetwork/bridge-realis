use async_trait::async_trait;
use realis_adapter::BridgeEvents;
use realis_bridge::TokenId;
use runtime::realis_bridge;
use secp256k1::SecretKey;
use sp_core::H160;
use web3::types::{Address, U256};

use slog::{error, info};
use utils::{accounts, contract, logger};

pub struct BscSender {
    // web3: web3::Web3<WebSocket>,
    // contract: Contract<WebSocket>,
    wallet_key: SecretKey,
}

impl BscSender {
    pub async fn new() -> BscSender {
        let wallet_key = accounts::realis("bsc-sender/res/accounts.key");

        BscSender {
            // web3,
            // contract,
            wallet_key,
        }
    }
}

#[async_trait]
impl BridgeEvents for BscSender {
    async fn on_transfer_token_to_bsc<'a>(&self, to: H160, value: u128) {
        let log = logger::new();

        let contract = contract::token_new(
            "wss://data-seed-prebsc-1-s1.binance.org:8545/",
        )
        .await;

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
            Ok(value) => info!(log, "Transaction success {:?}", value),
            Err(err) => error!(log, "Transaction fail {:?}", err),
        }
    }

    async fn on_transfer_nft_to_bsc<'a>(&self, to: H160, token_id: TokenId) {
        let log = logger::new();

        let contract =
            contract::nft_new("wss://data-seed-prebsc-1-s1.binance.org:8545/")
                .await;
        // Convert arguments
        // let from: Address =
        // Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").
        // unwrap();
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
            Ok(value) => info!(log, "Transaction success {:?}", value),
            Err(err) => error!(log, "Transaction fail {:?}", err),
        }
    }
}
