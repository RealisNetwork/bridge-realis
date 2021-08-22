// use log::{error, info};
use log::{error, trace};
use primitive_types::{U256, H256};
use realis_primitives::{Basic, Rarity};
use runtime::AccountId;
use secp256k1::SecretKey;
use sp_core::H160;
use std::{fs, path::Path, str::FromStr};
use utils::contract;
use web3::types::{Address};
use web3::Error;

pub struct BscSender {}

impl BscSender {
    fn read_file_for_secret_key<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }

    pub async fn send_token_to_bsc(from: AccountId, to: H160, amount: u128) -> Result<web3::types::H256, Error> {
        println!(
            "Bsc-sender send_token_to_bsc: {} => {}, ({})",
            from, to, amount
        );

        let wallet_key =
            BscSender::read_file_for_secret_key("./bsc-sender/res/accounts.key");

        let contract = contract::token_new().await;

        // Convert arguments
        let from = from.to_string();
        let to = Address::from(to.0);
        let amount = amount * 100_000_000;

        // Send transaction
        let result = contract
            .signed_call_with_confirmations(
                "transferFromRealis",
                (from, to, amount),
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;
        // View on result
        match result {
            Ok(value) => {
                println!("Transaction success {:?}", value);
                Ok(value.transaction_hash)
            },
            Err(err) => {
                println!("Transaction fail {:?}", err);
                Err(err)
            },
        }
    }

    pub async fn send_nft_to_bsc(
        from: AccountId,
        to: H160,
        token_id: U256,
        token_type: Basic,
        rarity: Rarity,
    ) -> Result<web3::types::H256, Error> {
        println!(
            "Bsc-sender send_nft_to_bsc: {} => {}, ({}, {}, {:?})",
            from, to, token_id, token_type, rarity
        );

        let wallet_key =
            BscSender::read_file_for_secret_key("./bsc-sender/res/accounts.key");
        trace!("Take account");
        let contract = contract::nft_new().await;

        // Convert arguments
        let from = from.to_string();
        let to = Address::from(to.0);

        let result = contract
            .signed_call_with_confirmations(
                "safeMint",
                (from, to, token_id.to_string(), token_type),
                web3::contract::Options::default(),
                1,
                &wallet_key,
            )
            .await;
        trace!("Take result {:?}", result);
        match result {
            Ok(value) => {
                println!("Transaction success {:?}", value);
                Ok(value.transaction_hash)
            },
            Err(err) => {
                error!("Transaction fail {:?}", err);
                Err(err)
            }
        }
    }

    pub async fn send_token_approve_from_realis_to_bsc(to: H160, amount: u128) {
        println!("Bsc-sender send_token_approve_to_bsc {}, ({})", to, amount);

        let wallet_key =
            BscSender::read_file_for_secret_key("./bsc-sender/res/accounts.key");

        let contract = contract::token_new().await;

        // Convert arguments
        let to = Address::from(to.0);

        // Send transaction
        let result = contract
            .signed_call_with_confirmations(
                "burnFrom",
                (to, amount),
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

    pub async fn send_nft_approve_from_realis_to_bsc(
        to: H160,
        token_id: U256,
        token_type: Basic,
    ) {
        println!(
            "Bsc-sender send_nft_approve_to_bsc: {}, ({}, {})",
            to, token_id, token_type
        );

        let wallet_key =
            BscSender::read_file_for_secret_key("/bsc-sender/res/accounts.key");

        let contract = contract::nft_new().await;

        let _to: Address = Address::from(to.0);

        // TODO: remove to_string() when web3 updates to 0.10 primitive-types
        let result = contract
            .signed_call_with_confirmations(
                "transferNftToRealisApproved",
                token_id.to_string(),
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
