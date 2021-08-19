use futures::{Stream, StreamExt};
use log::{error, info};
use primitives::{Error, RealisRequest, Request, ResponderRequest};
use ratsio::{RatsioError, StanClient, StanMessage, StanOptions};

use bsc_sender::BscSender;
use realis_sender::RealisSender;
use tokio::sync::mpsc::Sender;
use utils::parse;

pub fn logger_setup() {
    use env_logger::Builder;
    use log::LevelFilter;
    use std::io::Write;

    let _ = Builder::new()
        .format(|buf, record| {
            writeln!(buf, "[{}] - {}", record.level(), record.args())
        })
        .filter(None, LevelFilter::Trace)
        .try_init();
}

/// # Panics
///
/// Message-broker for geting requests from site
pub async fn message_broker(sender: Sender<Request>) -> Result<(), RatsioError> {
    let mut subscription = sub_stan().await;

    while let Some(message) = subscription.next().await {
        match parse(&message) {
            Ok(request) => {
                let send_result = sender.send(request).await;
                match send_result {
                    Ok(_) => {}
                    Err(error) => error!("Send to channel error: {:?}", error),
                }
            }
            Err(error) => error!("{:?}", error),
        }
    }
    Ok(())
}

pub async fn listen() {
    let mut subscription = sub_stan().await;

    while let Some(message) = subscription.next().await {
        match parse(&message) {
            Ok(request) => match request {
                Request::Realis(RealisRequest::TransferTokenToBSC(
                    raw_request,
                )) => {
                    BscSender::send_token_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.amount,
                    )
                    .await;
                }
                Request::Realis(RealisRequest::TransferNftToBSC(raw_request)) => {
                    BscSender::send_nft_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                    )
                    .await;
                }
                Request::Realis(RealisRequest::TransferTokenToRealis(
                    raw_request,
                )) => {
                    RealisSender::send_token_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.amount,
                    );
                }
                Request::Realis(RealisRequest::TransferNftToRealis(
                    raw_request,
                )) => {
                    RealisSender::send_nft_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                        raw_request.params.rarity.parse().unwrap(),
                    );
                }
            },
            Err(error) => error!("{:?}", error),
        }
    }
}

async fn sub_stan() -> impl Stream<Item = StanMessage> {
    // Create stan options
    let client_id = "realis-bridge".to_string();
    let opts = StanOptions::with_options(
        "localhost:4222",
        "test-cluster",
        &client_id[..],
    );
    // Create STAN client
    let stan_client = StanClient::from_options(opts).await.unwrap();

    // Subscribe to STAN subject 'foo'
    stan_client
        .subscribe("realis-bridge", None, None)
        .await
        .unwrap()
        .1
}

/// # Errors
pub fn parse(message: &StanMessage) -> Result<Request, serde_json::Error> {
    // Convert message to string
    let message_string =
        String::from_utf8_lossy(message.payload.as_ref()).into_owned();
    // Convert to json value object
    let raw_request: Result<Request, serde_json::Error> =
        serde_json::from_str(&message_string);
    raw_request
}
