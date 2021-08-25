use std::sync::mpsc::channel as sync_chan;

use bsc_sender::BscSender;
use codec::Decode;
use futures::{
    channel::mpsc::{unbounded as async_chan, UnboundedReceiver as AsyncRx},
    StreamExt as _,
};
use log::{error, info, warn};
use primitive_types::H256;
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H256 as Hash};
use substrate_api_client::{utils::FromHexString, Api};
use web3;

pub struct RealisAdapter(AsyncRx<String>);

impl RealisAdapter {
    /// # Panics
    ///
    /// Connection to Realis.Network for transfers
    #[must_use]
    pub fn new(url: &str) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();

        let (async_tx, async_rx) = async_chan();
        std::thread::spawn(move || {
            let (sync_tx, sync_rx) = sync_chan();

            api.subscribe_events(sync_tx).unwrap();

            loop {
                match sync_rx.recv() {
                    Ok(event) => {
                        if async_tx.unbounded_send(event).is_err() {
                            println!("Event handler was dropped");
                            break;
                        }
                    }
                    Err(error) => {
                        println!("Error while listen {:?}", error);
                        break;
                    }
                }
            }
        });

        Self(async_rx)
    }

    pub async fn listen(&mut self) {
        while let Some(event_str) = self.0.next().await {
            let events = parse_events(event_str);
            for event in events {
                match handle_event(event).await {
                    Ok(Some(result)) => info!("{:?}", result),
                    Ok(None) => warn!("Got unknown error"), 
                    Err(error) => error!("{:?}", error)
                }
            }
        }
    }
}

fn parse_events(event_str: String) -> Vec<system::EventRecord<Event, Hash>> {
    let unhex = Vec::from_hex(event_str).unwrap();
    Decode::decode(&mut unhex.as_slice()).unwrap()
}

async fn handle_event(
    event: system::EventRecord<Event, Hash>,
) -> Result<Option<H256>, web3::Error> {
    if let Event::RealisBridge(bridge_event) = event.event {
        match bridge_event {
            realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                info!(
                    "Realis-adapter handled TransferTokenToBSC: {} => {}, {}",
                    from,
                    to,
                    value
                );
                let tx_bsc =
                    BscSender::send_token_to_bsc(from.clone(), to, value).await;
                match tx_bsc {
                    Ok(tx_hash) => {
                        info!("Transaction send: {:?}", tx_hash);
                        return Ok(Some(tx_hash));
                    }
                    Err(error) => {
                        info!("Transaction fail: {:?}", error);
                        return Err(error);
                    }
                }
            }
            realis_bridge::Event::TransferNftToBSC(
                from,
                to,
                token_id_from_mint,
                token_type,
                rarity,
            ) => {
                info!(
                    "Realis-adapter handled TransferNftToBSC: {} => {}, {}",
                    from, to, token_id_from_mint
                );
                let token_id_str = &token_id_from_mint.to_string();
                let token_id =
                    primitive_types::U256::from_dec_str(token_id_str).unwrap();
                let tx_bsc = BscSender::send_nft_to_bsc(
                    from, to, token_id, token_type, rarity,
                )
                .await;
                match tx_bsc {
                    Ok(tx_hash) => {
                        info!("Transaction send: {:?}", tx_hash);
                        return Ok(Some(tx_hash));
                    }
                    Err(error) => {
                        error!("Transaction fail: {:?}", error);
                        return Err(error);
                    }
                }
            }
            realis_bridge::Event::TransferTokenToRealis(from, to, amount) => {
                // This event appears when tokens transfer from bsc to
                // realis And realis blockchain
                // confirmed this transfer
                info!(
                    "Realis-adapter handled TransferTokenToRealis: \
                        {} => {}, {}",
                    from, to, amount
                );
                let tx_bsc =
                    BscSender::send_token_approve_from_realis_to_bsc(from, amount)
                        .await;
                match tx_bsc {
                    Ok(tx_hash) => {
                        info!("Transaction send: {:?}", tx_hash);
                        return Ok(Some(tx_hash));
                    }
                    Err(error) => {
                        error!("Transaction fail: {:?}", error);
                        return Err(error);
                    }
                }
            }
            realis_bridge::Event::TransferNftToRealis(
                from,
                to,
                token_id_from_mint,
                token_type,
                rarity,
            ) => {
                // This event appears when nft transfer from bsc to realis
                // And realis blockchain confirmed this transfer
                info!(
                    "Realis-adapter handled TransferNftToRealis: \
                        {} => {}, {}, {:?}",
                    from, to, token_id_from_mint, rarity
                );
                let token_id_str = &token_id_from_mint.to_string();
                let token_id =
                    primitive_types::U256::from_dec_str(token_id_str).unwrap();
                let tx_bsc = BscSender::send_nft_approve_from_realis_to_bsc(
                    from, token_id, token_type, rarity,
                )
                .await;
                match tx_bsc {
                    Ok(tx_hash) => {
                        info!("Transaction send: {:?}", tx_hash);
                        return Ok(Some(tx_hash));
                    }
                    Err(error) => {
                        error!("Transaction fail: {:?}", error);
                        return Err(error);
                    }
                }
            }
            _ => Ok(None)
        }
    } 
    else {
        return Ok(None);
    }
}
