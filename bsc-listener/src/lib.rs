use ethabi::{ethereum_types::U256, ParamType};
use log::{error, info, LevelFilter};
use primitives::{
    events::{BscEventType, TransferNftToRealis, TransferTokenToRealis},
    types::BlockNumber,
    Error,
};
use realis_primitives::TokenId;
use runtime::AccountId;
use rust_lib::logger;
use serde::Deserialize;
use serde_json::Value;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{channel, unbounded_channel, Sender, UnboundedReceiver, UnboundedSender};
use web3::{self, futures::StreamExt, types::Address};

pub struct BlockListener {
    url: String,
    tx: Sender<BscEventType>,
    status: Arc<AtomicBool>,
}

impl BlockListener {
    /// # Errors
    pub fn new(url: String, tx: Sender<BscEventType>, status: Arc<AtomicBool>) -> Self {
        Self { url, tx, status }
    }

    pub async fn listen(self) {
        let ws = web3::transports::WebSocket::new(&self.url).await.unwrap();
        let web3 = web3::Web3::new(ws.clone());
        // Stream
        let mut sub = web3.eth_subscribe().subscribe_new_heads().await.unwrap();

        info!("Got subscription id: {:?}", sub.id());

        while let Some(value) = sub.next().await {
            let block = value.unwrap();
            let some = web3
                .eth()
                .block_with_txs(web3::types::BlockId::Hash(block.hash.unwrap()))
                .await
                .unwrap()
                .unwrap();

            for transaction in some.transactions {
                match transaction.to {
                    Some(account) => {
                        if account == Address::from_str("0x1c43b4253c33d246ad27e710d949a8d8b62a2c73").unwrap() {
                            match transaction.from {
                                Some(account_from) => {
                                    if account_from
                                        != Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap()
                                    {
                                        info!("[Transactions] - {:?}", transaction);
                                        info!("Transaction hash: {:?}", transaction.hash);

                                        let tx = web3
                                            .eth()
                                            .transaction_receipt(transaction.hash)
                                            .await
                                            .unwrap()
                                            .unwrap();
                                        info!("{:?}", tx);
                                        for log in tx.logs {
                                            info!(
                                                "{:?}",
                                                ethabi::decode(
                                                    &[ParamType::String, ParamType::Uint(256)],
                                                    &log.data.0
                                                )
                                            );
                                            let info = ethabi::decode(
                                                &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                                                &log.data.0,
                                            )
                                            .unwrap();
                                            let json: Result<Value, serde_json::error::Error> =
                                                serde_json::to_value(&info[0].to_string());
                                            match json {
                                                Ok(value) => {
                                                    let account_id: AccountId =
                                                        Deserialize::deserialize(value).unwrap();
                                                    let amount =
                                                        U256::from(info[1].clone().into_uint().unwrap()).as_u128();

                                                    info!("{:?}", account_id);
                                                    self.tx
                                                        .send(BscEventType::TransferTokenToRealisSuccess(
                                                            TransferTokenToRealis {
                                                                block: transaction.block_number,
                                                                hash: transaction.hash,
                                                                from: account_from,
                                                                to: account_id,
                                                                amount,
                                                            },
                                                            transaction.hash,
                                                            transaction.block_number,
                                                        ))
                                                        .await;
                                                }
                                                Err(error) => error!("Cannot parse Realis account: {:?}", error),
                                            }
                                        }
                                    }
                                }
                                None => (),
                            }
                        } else if account
                            == Address::from_str("0x2197dc003055ceb84f6b27c73c17a7f4d39ada4a").unwrap()
                        {
                            match transaction.from {
                                Some(account_from) => {
                                    if account_from
                                        != Address::from_str("0x12815AF79eE96Ef72167C3746a4aD251105F1981").unwrap()
                                    {
                                        info!("[Transactions] - {:?}", transaction);
                                        info!("Transaction hash: {:?}", transaction.hash);

                                        let tx = web3
                                            .eth()
                                            .transaction_receipt(transaction.hash)
                                            .await
                                            .unwrap()
                                            .unwrap();
                                        info!("{:?}", tx);
                                        for log in tx.logs {
                                            info!(
                                                "{:?}",
                                                ethabi::decode(
                                                    &[ParamType::String, ParamType::Uint(256)],
                                                    &log.data.0
                                                )
                                            );
                                            let info = ethabi::decode(
                                                &[ParamType::String, ParamType::Uint(256), ParamType::Address],
                                                &log.data.0,
                                            )
                                            .unwrap();
                                            let json: Value = serde_json::from_str(&info[0].to_string()).unwrap();
                                            let account_id: AccountId = Deserialize::deserialize(json).unwrap();
                                            let token_id = TokenId::from_str(&info[0].to_string()).unwrap();
                                            info!("{:?}", account_id);
                                            self.tx
                                                .send(BscEventType::TransferNftToRealisSuccess(
                                                    TransferNftToRealis {
                                                        block: transaction.block_number,
                                                        hash: transaction.hash,
                                                        from: account_from,
                                                        dest: account_id,
                                                        token_id,
                                                    },
                                                    transaction.hash,
                                                    transaction.block_number,
                                                ))
                                                .await;
                                        }
                                    }
                                }
                                None => (),
                            }
                        }
                    }
                    None => (),
                }
            }
        }
        sub.unsubscribe().await.unwrap();
    }
}
