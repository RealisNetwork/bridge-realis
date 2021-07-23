use sp_core::{sr25519, H160};
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use std::str::FromStr;
use secp256k1::SecretKey;
use runtime::realis_bridge;
use realis_bridge::TokenId;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use realis_adapter::BridgeEvents;
use web3::transports::WebSocket;
use serde::Deserialize;
use serde_json::value::Value;
use std::fs;
use hex_literal::hex;
use log::{info, warn};

// #[derive(Deserialize, Debug)]
// struct Key {
//     secret_key: String
// }

// #[tokio::main]
// async fn main() -> web3::Result<()> {
//     let _ =env_logger::try_init();
//     let wss = web3::transports::WebSocket::new("wss://data-seed-prebsc-1-s1.binance.org:8545/").await?;
//     let web3 = web3::Web3::new(wss);
//
//     let json_abi = include_bytes!("../res/BEP20.abi");
//     // TODO add file like env for private keys
//     let address: Address = Address::from_str("0x0db8499bb62772e805af78fc918ee8c8cd6a2859").unwrap();
//     let key = read_file_for_secret_key("./res/accounts.key");
//     println!("{:?}", key);
//     let registry_contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();
//     let from: Address = Address::from_str("0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943").unwrap();
//     let to: Address = Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap();
//     let value = U256::from(10_000_000_000_000_000_000_000_000 as u128);
//     // let params = Tokenizable::into_token([from, to]);
//     let params = (to, value, );
//
//     let signed_tx = registry_contract
//         .signed_call_with_confirmations("transfer", params, Default::default(), 1, &key)
//         .await;
//     println!("Transaction hash: {:?}", signed_tx.unwrap().transaction_hash);
//     Ok(())
// }

pub struct BscSender {
    web3: web3::Web3<WebSocket>,
    contract: Contract<WebSocket>,
    wallet_key: SecretKey
}



impl BscSender {

    pub async fn new(url: &str) -> BscSender {
        let wss = WebSocket::new(url).await.unwrap();
        let web3 = web3::Web3::new(wss);

        let address: Address = Address::from_str("0x0db8499bb62772e805af78fc918ee8c8cd6a2859").unwrap();
        let json_abi = include_bytes!("../res/BEP20.abi");
        let contract = Contract::from_json(web3.eth(), address, json_abi).unwrap();


        let wallet_key = BscSender::read_file_for_secret_key("bsc-sender/res/accounts.key");

        BscSender {
            web3,
            contract,
            wallet_key
        }
    }

    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}

impl BridgeEvents for BscSender {
    fn on_transfer_token_to_bsc(&self, to: &H160, value: &u128) {
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(*value);

        let a = self.contract
            .signed_call_with_confirmations("transfer", (to, value), Default::default(), 1, &self.wallet_key);
    }

    fn on_transfer_nft_to_bsc(&self, to: &H160, token_id: &TokenId) {

    }

    fn on_transfer_token_to_realis(&self, to: &runtime::AccountId, value: &u128) {

    }

    fn on_transfer_nft_to_realis(&self, to: &runtime::AccountId, token_id: &U256) {

    }
}