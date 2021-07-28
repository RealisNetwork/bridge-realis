use async_trait::async_trait;
use codec::Decode;
use realis_bridge::TokenId;
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H160, H256 as Hash};
use std::sync::mpsc::{channel, Receiver};
use substrate_api_client::utils::FromHexString;
use substrate_api_client::Api;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

use slog::Drain;

pub struct RealisAdapter<T: BridgeEvents> {
    // events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T,
}

impl<T: BridgeEvents> RealisAdapter<T> {
    pub fn new(url: String, event_handler: T) -> Self {
        // Connect to api
        let api = Api::<sr25519::Pair>::new(format!("wss://{}", url)).unwrap();
        // Create channels
        let (events_in, events_out) = channel();
        // Subscribe on events
        api.subscribe_events(events_in.clone()).unwrap();

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
        return Vec::<system::EventRecord<Event, Hash>>::decode(&mut er_enc)
            .unwrap();
    }

    async fn process_event(&self, event: &system::EventRecord<Event, Hash>) {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());

        match &event.event {
            Event::RealisBridge(bridge_event) => match bridge_event {
                realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                    self.event_handler
                        .on_transfer_token_to_bsc(&to, value)
                        .await;
                    info!(log, "From {}", from);
                    info!(log, "From {}", to);
                    info!(log, "From {}", value);
                }
                realis_bridge::Event::TransferNftToBSC(from, to, token_id) => {
                    self.event_handler
                        .on_transfer_nft_to_bsc(&to, &token_id)
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
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());

        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events =
                        RealisAdapter::<T>::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event).await;
                    }
                }
                Err(error) => error!(log, "Error while listen {:?}", error)
            }
        }
    }
}

#[async_trait]
pub trait BridgeEvents {
    async fn on_transfer_token_to_bsc<'a>(&self, to: &H160, value: &u128);
    async fn on_transfer_nft_to_bsc<'a>(&self, to: &H160, token_id: &TokenId);
}
