use async_trait::async_trait;
use logger::logger::{log, Type};
use realis_adapter::BridgeEvents;
use realis_bridge::TokenId;
use runtime::realis_bridge;
use secp256k1::SecretKey;
use sp_core::H160;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use web3::contract::Contract;
use web3::transports::WebSocket;
use web3::types::{Address, U256};


pub struct BscSender {
    // web3: web3::Web3<WebSocket>,
    // contract: Contract<WebSocket>,
    wallet_key: SecretKey,
}

impl BscSender {
    pub async fn new() -> BscSender {
        let wallet_key = BscSender::read_file_for_secret_key("./../bsc-sender/res/accounts.key");

        BscSender {
            // web3,
            // contract,
            wallet_key,
        }
    }

    async fn get_connection(url: &str) -> Contract<WebSocket> {
        // Connect to bsc
        let mut wss = WebSocket::new(url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    log(Type::Error, String::from("Cannot connect"), &error);
                    log(Type::Info, String::from("Try to reconnect"), &());
                    wss = WebSocket::new(url).await;
                }
            }
        }

        let web3 = web3::Web3::new(wss.unwrap());

        let address: Address =
            Address::from_str("0x0db8499bb62772e805af78fc918ee8c8cd6a2859").unwrap();
        let json_abi = include_bytes!("../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}

#[async_trait]
impl BridgeEvents for BscSender {
    async fn on_transfer_token_to_bsc<'a>(&self, to: &H160, value: &u128) {
        let contract = BscSender::get_connection("wss://data-seed-prebsc-1-s1.binance.org:8545/").await;
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(*value) * 100_000_000;

        let result = contract
            .signed_call_with_confirmations(
                "transfer",
                (to, value),
                Default::default(),
                1,
                &self.wallet_key,
            )
            .await;

        match result {
            Ok(value) => log(
                Type::Success,
                String::from("Transaction hash"),
                &value.transaction_hash,
            ),
            Err(err) => log(Type::Error, String::from("Transaction fail"), &err),
        }
    }

    async fn on_transfer_nft_to_bsc<'a>(&self, to: &H160, token_id: &TokenId) {
        let contract = BscSender::get_connection("wss://data-seed-prebsc-1-s1.binance.org:8545/").await;
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(token_id);

        let result = contract
            .signed_call_with_confirmations(
                "transfer",
                (to, value),
                Default::default(),
                1,
                &self.wallet_key,
            )
            .await;

        match result {
            Ok(value) => log(
                Type::Success,
                String::from("Transaction hash"),
                &value.transaction_hash,
            ),
            Err(err) => log(Type::Error, String::from("Transaction fail"), &err),
        }
    }
}
