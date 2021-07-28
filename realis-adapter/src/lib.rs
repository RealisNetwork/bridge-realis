use async_trait::async_trait;
use codec::Decode;
use realis_bridge::TokenId;
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H160, H256 as Hash};
use std::sync::mpsc::{channel, Receiver};
use substrate_api_client::{utils::FromHexString, Api};

use slog::{error, info, warn};
use utils::logger;

pub struct RealisAdapter<T: BridgeEvents> {
    // events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T,
}

impl<T: BridgeEvents> RealisAdapter<T> {
    /// # Panics
    ///
    /// Connect to Realis.Network for transfers
    pub fn new(url: &str, event_handler: T) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();
        // Create channels
        let (events_in, events_out) = channel();
        // Subscribe on events
        api.subscribe_events(events_in).unwrap();

        RealisAdapter {
            // events_in,
            events_out,
            event_handler,
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
        let log = logger::new();

        match &event.event {
            Event::RealisBridge(bridge_event) => match bridge_event {
                realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                    self.event_handler
                        .on_transfer_token_to_bsc(*to, *value)
                        .await;
                    info!(log, "From {}", from);
                    info!(log, "From {}", to);
                    info!(log, "From {}", value);
                }
                realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
                    self.event_handler
                        .on_transfer_nft_to_bsc(*to, *token_id)
                        .await;
                    info!(log, "From {}", from);
                    info!(log, "From {}", to);
                    info!(log, "From {}", token_id);
                }
                realis_bridge::Event::TransferTokenToRealis(to, value) => {
                    info!(log, "From {}", to);
                    info!(log, "From {}", value);
                }
                realis_bridge::Event::TransferNftToRealis(to, token_id) => {
                    info!(log, "From {}", to);
                    info!(log, "From {}", token_id);
                }
                _ => warn!(log, "Unsupported event {:?}", event.event),
            },
            _ => warn!(log, "Unsupported event {:?}", event.event),
        }
    }

    // Add bsc sender as argument
    pub async fn listener(&self) {
        let log = logger::new();

        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events =
                        RealisAdapter::<T>::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event).await;
                    }
                }
                Err(error) => error!(log, "Error while listen {:?}", error),
            }
        }
    }
}

#[async_trait]
pub trait BridgeEvents {
    async fn on_transfer_token_to_bsc<'a>(&self, to: H160, value: u128);
    async fn on_transfer_nft_to_bsc<'a>(&self, to: H160, token_id: TokenId);
}
