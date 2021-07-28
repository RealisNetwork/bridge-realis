use async_trait::async_trait;
use realis_adapter::BridgeEvents;
use realis_bridge::TokenId;
use runtime::realis_bridge;
use secp256k1::SecretKey;
use sp_core::H160;
use std::{fs, path::Path, str::FromStr};
use web3::{
    contract::Contract,
    transports::WebSocket,
    types::{Address, U256},
};

use slog::{error, info};
use utils::logger;

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

    async fn get_connection(url: &str) -> Contract<WebSocket> {
        let log = logger::new();
        // Connect to bsc
        let mut wss = WebSocket::new(url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!(log, "Cannot connect {:?}", error);
                    info!(log, "Try reconnect");
                    wss = WebSocket::new(url).await;
                }
            }
        }

        let web3 = web3::Web3::new(wss.unwrap());

        let address: Address =
            Address::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef")
                .unwrap();
        let json_abi = include_bytes!("../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    async fn get_connection_nft(url: &str) -> Contract<WebSocket> {
        let log = logger::new();
        // Connect to bsc
        let mut wss = WebSocket::new(url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!(log, "Cannot connect {:?}", error);
                    info!(log, "Try reconnect");
                    wss = WebSocket::new(url).await;
                }
            }
        }

        let web3 = web3::Web3::new(wss.unwrap());

        let address: Address =
            Address::from_str("0x8A19360f2EC953b433D92571120bb5ef755b3d17")
                .unwrap();
        let json_abi = include_bytes!("../res/BEP721.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}

#[async_trait]
impl BridgeEvents for BscSender {
    async fn on_transfer_token_to_bsc<'a>(&self, to: H160, value: u128) {
        let log = logger::new();

        let contract = BscSender::get_connection(
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

        let contract = BscSender::get_connection_nft(
            "wss://data-seed-prebsc-1-s1.binance.org:8545/",
        )
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
