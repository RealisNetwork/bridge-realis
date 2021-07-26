use sp_core::H160;
use web3::contract::Contract;
use web3::types::{Address, U256};
use web3::transports::WebSocket;
use std::str::FromStr;
use secp256k1::SecretKey;
use runtime::realis_bridge;
use realis_bridge::TokenId;
use std::path::Path;
use realis_adapter::BridgeEvents;
use std::fs;
use async_trait::async_trait;
use logger::logger::{log, Type};

pub struct BscSender {
    // web3: web3::Web3<WebSocket>,
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


        let wallet_key = BscSender::read_file_for_secret_key("./bsc-sender/res/accounts.key");

        BscSender {
            // web3,
            contract,
            wallet_key
        }
    }

    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}

#[async_trait]
impl BridgeEvents for BscSender {
    async fn on_transfer_token_to_bsc<'a>(&self, to: &H160, value: &u128) {
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(*value) * 100_000_000;

        let result = self.contract
            .signed_call_with_confirmations("transfer", (to, value), Default::default(), 1, &self.wallet_key)
            .await;

        match result {
            Ok(value) => log(Type::Success, String::from("Transaction hash"), &value.transaction_hash),
            Err(err) => log(Type::Error, String::from("Transaction fail"), &err)
        }
    }

    async fn on_transfer_nft_to_bsc<'a>(&self, to: &H160, token_id: &TokenId) {
        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(token_id);
        println!("Account BSC: {:?}", to);
        println!("Value: {:?}", value);

        let result = self.contract
            .signed_call_with_confirmations("transfer", (to, value), Default::default(), 1, &self.wallet_key)
            .await;

        match result {
            Ok(value) => log(Type::Success, String::from("Transaction hash"), &value.transaction_hash),
            Err(err) => log(Type::Error, String::from("Transaction fail"), &err)
        }
    }
}