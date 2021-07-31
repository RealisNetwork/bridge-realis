// use bridge_events::Events;
use codec::Decode;
// use log::{error, info, warn};
// use realis_bridge::TokenId;
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H256 as Hash};
use std::sync::mpsc::{channel, Receiver};
use substrate_api_client::{utils::FromHexString, Api};

use bsc_sender::BscSender;

pub struct RealisAdapter {
    channel_from_realis: Receiver<String>,
}

impl RealisAdapter {
    /// # Panics
    ///
    /// Conection to Realis.Network for transfers
    #[must_use]
    pub fn new(url: &str) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();
        // Create channels
        let (events_in, channel_from_realis) = channel();
        // Subscribe on events
        api.subscribe_events(events_in).unwrap();

        RealisAdapter {
            channel_from_realis,
        }
    }

    fn parse_events_str(
        event_str: String,
    ) -> Vec<system::EventRecord<Event, Hash>> {
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc).unwrap()
    }

    async fn process_event(&self, event: &system::EventRecord<Event, Hash>) {
        if let Event::RealisBridge(bridge_event) = &event.event {
            match bridge_event {
                realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                    println!(
                      "Realis-adapter handled TransferTokenToBSC: {} => {}, {}",
                      from, to, value
                    );
                    BscSender::send_token_to_bsc(from.clone(), *to, *value)
                        .await;
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
                        *to,
                        *token_id,
                        *token_type,
                    )
                    .await;
                }
                realis_bridge::Event::TransferTokenToRealis(to, value) => {
                    // This event appears when tokens transfer from bsc to
                    // realis And realis blockchain
                    // confirmed this transfer
                    println!(
                      "Realis-adapter handled TransferTokenToRealis: => {}, {}",
                      to, value
                    );
                    // TODO impl
                }
                // TODO receive token_type
                realis_bridge::Event::TransferNftToRealis(to, token_id) => {
                    // This event appears when nft transfer from bsc to realis
                    // And realis blockchain confirmed this transfer
                    println!(
                        "Realis-adapter handled TransferNftToRealis: => {}, {}",
                        to, token_id
                    );
                    // TODO impl
                }
                _ => println!(
                    "Unsupported event in Bridge-pallet {:?}",
                    event.event
                ),
            }
        } else {
            println!("Unsupported event {:?}", event.event);
        }
    }

    pub async fn listen(&self) {
        loop {
            match self.channel_from_realis.recv() {
                Ok(event_str) => {
                    let events = RealisAdapter::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event).await;
                    }
                }
                Err(error) => println!("Error while listen {:?}", error),
            }
        }
    }
}
