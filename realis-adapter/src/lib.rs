use std::sync::mpsc::channel as sync_chan;

use bsc_sender::BscSender;
use codec::Decode;
// use log::{error, info, warn};
use futures::{
    channel::mpsc::{unbounded as async_chan, UnboundedReceiver as AsyncRx},
    StreamExt as _,
};
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H256 as Hash};
use substrate_api_client::{utils::FromHexString, Api};

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
                handle_event(event).await;
            }
        }
    }
}

fn parse_events(event_str: String) -> Vec<system::EventRecord<Event, Hash>> {
    let unhex = Vec::from_hex(event_str).unwrap();
    let mut er_enc = unhex.as_slice();
    Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc).unwrap()
}

async fn handle_event(event: system::EventRecord<Event, Hash>) {
    if let Event::RealisBridge(bridge_event) = event.event {
        match bridge_event {
            realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                println!(
                    "Realis-adapter handled TransferTokenToBSC: {} => {}, {}",
                    from, to, value
                );
                BscSender::send_token_to_bsc(from.clone(), to, value).await;
            }
            realis_bridge::Event::TransferNftToBSC(
                from,
                to,
                token_id,
                token_type,
            ) => {
                println!(
                    "Realis-adapter handled TransferNftToBSC: {} => {}, {}",
                    from, to, token_id
                );
                BscSender::send_nft_to_bsc(
                    from.clone(),
                    to,
                    token_id,
                    token_type,
                )
                .await;
            }
            realis_bridge::Event::TransferTokenToRealis(from, to, amount) => {
                // This event appears when tokens transfer from bsc to
                // realis And realis blockchain
                // confirmed this transfer
                println!(
                    "Realis-adapter handled TransferTokenToRealis: \
                        {} => {}, {}",
                    from, to, amount
                );
                BscSender::send_token_approve_from_realis_to_bsc(from, amount)
                    .await;
            }
            realis_bridge::Event::TransferNftToRealis(
                from,
                to,
                token_id,
                token_type,
            ) => {
                // This event appears when nft transfer from bsc to realis
                // And realis blockchain confirmed this transfer
                println!(
                    "Realis-adapter handled TransferNftToRealis: \
                        {} => {}, {}",
                    from, to, token_id
                );
                BscSender::send_nft_approve_from_realis_to_bsc(
                    from, token_id, token_type,
                )
                .await;
            }
            _ => {
                // println!(
                //     "Unsupported event in Bridge-pallet {:?}",
                //     event.event
                // )
            }
        }
    } else {
        // println!("Unsupported event {:?}", event.event);
    }
}
