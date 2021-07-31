// use log::{error, info};
use realis_primitives::{Basic, TokenId};
use runtime::AccountId;
use secp256k1::SecretKey;
use sp_core::H160;
use std::{fs, path::Path, str::FromStr};
use utils::contract;
use web3::types::{Address, U256};

pub struct BscSender {}

impl BscSender {
    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }

    pub async fn send_token_to_bsc(from: AccountId, to: H160, amount: u128) {
        println!(
            "Bsc-sender send_token_to_bsc: {} => {}, ({})",
            from, to, amount
        );

        let wallet_key = BscSender::read_file_for_secret_key(
            "./bsc-sender/res/accounts.key",
        );

        let contract = contract::token_new().await;

        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(amount) * 100_000_000;

        // Send transaction
        let result = contract
            .signed_call_with_confirmations(
                "transferToRealis",
                (to, value), // TODO add from and change contract
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;
        // View on result
        match result {
            Ok(value) => println!("Transaction success {:?}", value),
            Err(err) => println!("Transaction fail {:?}", err),
        }
    }

    pub async fn send_nft_to_bsc(
        from: AccountId,
        to: H160,
        token_id: TokenId,
        token_type: Basic,
    ) {
        println!(
            "Bsc-sender send_nft_to_bsc: {} => {}, ({}, {})",
            from, to, token_id, token_type
        );

        let wallet_key = BscSender::read_file_for_secret_key(
            "./bsc-sender/res/accounts.key",
        );

        let contract = contract::nft_new().await;

        let to: Address = Address::from(to.0);

        let result = contract
            .signed_call_with_confirmations(
                "safeMint",
                (to, token_id, token_type),
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;

        match result {
            Ok(value) => println!("Transaction success {:?}", value),
            Err(err) => println!("Transaction fail {:?}", err),
        }
    }

    pub async fn send_token_approve_to_bsc(to: H160, amount: u128) {
        println!("Bsc-sender send_token_approve_to_bsc {}, ({})", to, amount);

        let wallet_key = BscSender::read_file_for_secret_key(
            "./bsc-sender/res/accounts.key",
        );

        let contract = contract::token_new().await;

        // Convert arguments
        let to: Address = Address::from(to.0);
        let value = U256::from(amount) * 100_000_000;
        // Send transaction
        let result = contract
            .signed_call_with_confirmations(
                "transfer",
                (to, value),
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;
        // View on result
        match result {
            Ok(value) => println!("Transaction success {:?}", value),
            Err(err) => println!("Transaction fail {:?}", err),
        }
    }

    pub async fn send_nft_approve_to_bsc(
        to: H160,
        token_id: TokenId,
        token_type: Basic,
    ) {
        println!(
            "Bsc-sender send_nft_to_bsc: {}, ({}, {})",
            to, token_id, token_type
        );

        let wallet_key = BscSender::read_file_for_secret_key(
            "./bsc-sender/res/accounts.key",
        );

        let contract = contract::nft_new().await;

        let to: Address = Address::from(to.0);

        let result = contract
            .signed_call_with_confirmations(
                "",
                (to, token_id),
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;

        match result {
            Ok(value) => println!("Transaction success {:?}", value),
            Err(err) => println!("Transaction fail {:?}", err),
        }
    }
}
