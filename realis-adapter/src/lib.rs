use async_trait::async_trait;
use codec::Decode;
use log::{error, info, warn};
use realis_bridge::TokenId;
use runtime::{realis_bridge, Event};
use sp_core::{sr25519, H160, H256 as Hash};
use std::sync::mpsc::{channel, Receiver};
use substrate_api_client::{utils::FromHexString, Api};

pub struct RealisAdapter<T: BridgeEvents> {
    // events_in: Sender<String>,
    events_out: Receiver<String>,
    event_handler: T,
}

impl<T: BridgeEvents> RealisAdapter<T> {
    /// # Panics
    ///
    /// Conection to Realis.Network for transfers
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
        if let Event::RealisBridge(bridge_event) = &event.event {
            match bridge_event {
                realis_bridge::Event::TransferTokenToBSC(from, to, value) => {
                    self.event_handler
                        .on_transfer_token_to_bsc(*to, *value)
                        .await;
                    info!(
                        "Handled TransferTokenToBSC: {} => {}, {}",
                        from, to, value
                    );
                }
                realis_bridge::Event::TransferNftToBSC(
                    from,
                    to,
                    token_id,
                    token_type,
                ) => {
                    self.event_handler
                        .on_transfer_nft_to_bsc(*to, *token_id, *token_type)
                        .await;
                    info!(
                        "Handled TransferNftToBSC: {} => {}, {}",
                        from, to, token_id
                    );
                }
                realis_bridge::Event::TransferTokenToRealis(to, value) => {
                    // This event appears when tokens transfer from bsc to realis
                    // And realis blockchain confirmed this transfer
                    info!(
                        "Handled TransferTokenToRealis: => {}, {}",
                        to, value
                    );
                }
                realis_bridge::Event::TransferNftToRealis(to, token_id) => {
                    // This event appears when nft transfer from bsc to realis
                    // And realis blockchain confirmed this transfer
                    info!(
                        "Handled TransferNftToRealis: => {}, {}",
                        to, token_id
                    );

                }
                _ => warn!("Unsupported event {:?}", event.event),
            }
        } else {
            warn!("Unsupported event {:?}", event.event);
        }
    }

    // Add bsc sender as argument
    pub async fn listener(&self) {
        loop {
            match self.events_out.recv() {
                Ok(event_str) => {
                    let events =
                        RealisAdapter::<T>::parse_events_str(event_str);
                    for event in &events {
                        self.process_event(event).await;
                    }
                }
                Err(error) => error!("Error while listen {:?}", error),
            }
        }
    }
}

#[async_trait]
pub trait BridgeEvents {
    async fn on_transfer_token_to_bsc<'a>(&self, to: H160, value: u128);
    async fn on_transfer_nft_to_bsc<'a>(
        &self,
        to: H160,
        token_id: TokenId,
        token_type: u8,
    );

}
