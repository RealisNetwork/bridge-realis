use ethabi::{Address, Uint};
use futures::join;
// use log::{error, info};
use realis_sender::RealisSender;
use runtime::AccountId;
use sp_core::{crypto::Ss58Codec, H160, U256};
use utils::contract;
use web3::{
    contract::{Contract, Result as web3Result},
    transports::WebSocket,
};

// TODO from struct to functions??? or find better solution
struct BSCListener(Contract<WebSocket>);

impl BSCListener {
    pub fn new(contract: Contract<WebSocket>) -> Self {
        Self(contract)
    }

    pub async fn listen_token(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(Address, String, Uint)>> =
                self.0.events("TransferToRealis", (), (), ()).await;
            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        // Log event
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (from, to, value) = event;
                        // Convert argument
                        let account_id = AccountId::from_ss58check(&to).unwrap();
                        // Log arguments
                        println!(
                            "TransferTokenToRealis: {:?} => {:?}, {:?}",
                            from, to, value
                        );
                        RealisSender::send_token_to_realis(
                            H160::from(from.0),
                            account_id,
                            value.as_u128(),
                        );
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_nft(&self) {
        loop {
            #[allow(clippy::type_complexity)]
            let logs: web3Result<
                Vec<(Address, String, Uint, u8, String)>,
            > = self.0.events("TransferNftToRealis", (), (), ()).await;

            match logs {
                Ok(events) => {
                    // Process all events
                    for (from, to, token_id_from_mint, basic, rarity) in events {
                        // Log event
                        println!(
                            "Get event {:?}",
                            (from, &to, token_id_from_mint, basic, &rarity)
                        );
                        // Convert arguments
                        let account_id = AccountId::from_ss58check(&to).unwrap();
                        // Log arguments
                        println!(
                            "TransferNftToRealis: {:?}, {:?}, {:?}",
                            to, token_id_from_mint, basic
                        );
                        let rarity_str = rarity.parse().unwrap();
                        let token_id = U256::from(token_id_from_mint.as_u32());
                        RealisSender::send_nft_to_realis(
                            H160::from(from.0),
                            account_id,
                            token_id,
                            basic,
                            rarity_str,
                        );
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_token_success(&self) {
        loop {
            let logs: web3::contract::Result<
                Vec<(Address, String, Address, Uint)>,
            > = self.0.events("TransferFromRealis", (), (), ()).await;
            match logs {
                Ok(events) => {
                    // Process all events
                    for event in events {
                        println!("Get event {:?}", event);
                        // Unpack event arguments
                        let (_, from, to, amount) = event;
                        // Convert argument
                        let account_id =
                            AccountId::from_ss58check(&from).unwrap();
                        // Log arguments
                        println!(
                            "TokenSuccessOnBsc: {:?} => {:?}, {:?}",
                            account_id, to, amount
                        );
                        RealisSender::send_token_approve_to_realis(
                            account_id,
                            amount.as_u128(),
                        )
                        .await;
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }

    pub async fn listen_nft_success(&self) {
        loop {
            let logs: web3::contract::Result<Vec<(String, Address, Uint, u8)>> =
                self.0.events("MintNftFromRealis", (), (), ()).await;
            match logs {
                Ok(events) => {
                    // Process all events
                    for (from, to, token_id_from_mint, basic) in events {
                        println!(
                            "Get event {:?}",
                            (&from, to, token_id_from_mint, basic)
                        );
                        // Convert argument
                        let account_id =
                            AccountId::from_ss58check(&from).unwrap();
                        let token_id =
                            sp_core::U256::from(token_id_from_mint.as_u32());
                        println!(
                            "TransferNftToRealis: {:?} => {:?}, {:?}, {:?}",
                            account_id, to, token_id, basic
                        );
                        RealisSender::send_nft_approve_to_realis_from_bsc(
                            account_id, token_id,
                        )
                        .await;
                    }
                }
                Err(error) => {
                    println!("Error while listen {:?}", error);
                }
            }
        }
    }
}

pub struct BSCAdapter {}

impl BSCAdapter {
    pub async fn listen() {
        let token_listener = BSCListener::new(contract::token_new().await);
        let nft_listener = BSCListener::new(contract::nft_new().await);
        let token_listener_success =
            BSCListener::new(contract::token_new().await);
        let nft_listener_success = BSCListener::new(contract::nft_new().await);

        join!(
            token_listener.listen_token(),
            nft_listener.listen_nft(),
            token_listener_success.listen_token_success(),
            nft_listener_success.listen_nft_success()
        );
    }
}
