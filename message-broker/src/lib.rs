use futures::{Stream, StreamExt};
use log::{error, info};
use primitives::{Error, RealisRequest, Request, ResponderRequest};
use ratsio::{StanClient, StanMessage, StanOptions};

use bsc_sender::BscSender;
use realis_sender::RealisSender;
use utils::{parse, parse::Request};

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
    let mut subscription = sub_stan().await?;

    while let Some(message) = subscription.1.next().await {
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
        match parse::convert_message(&message) {
            Ok(request) => match request {
                Request::TransferFromRealis {
                    account_id,
                    bsc_account,
                    amount,
                    ..
                } => {
                    BscSender::send_token_to_bsc(
                        account_id,
                        bsc_account,
                        amount.as_u128(),
                    )
                    .await;
                }
                Request::TransferFromRealisNft {
                    account_id,
                    bsc_account,
                    token_id,
                    token_type,
                    ..
                } => {
                    BscSender::send_nft_to_bsc(
                        account_id,
                        bsc_account,
                        token_id,
                        token_type,
                    )
                    .await;
                }
                Request::SendToRealis {
                    bsc_account,
                    account_id,
                    amount,
                    ..
                } => {
                    RealisSender::send_token_to_realis(
                        bsc_account,
                        &account_id,
                        amount.as_u128(),
                    );
                }
                Request::SendToRealisNft {
                    account_id,
                    bsc_account,
                    token_id,
                    token_type,
                    rarity,
                    ..
                } => {
                    RealisSender::send_nft_to_realis(
                        bsc_account,
                        &account_id,
                        token_id,
                        token_type,
                        rarity,
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
pub fn parse(message: &StanMessage) -> Result<Request, Error> {
    // Convert message to string
    let message_string =
        String::from_utf8_lossy(message.payload.as_ref()).into_owned();
    // Convert to json value object
    let raw_request: Result<DBRequest, serde_json::Error> =
        serde_json::from_str(&message_string);
    match raw_request {
        Ok(raw_request) => Ok(Request::DB(raw_request)),
        Err(_) => Err(Error::Parse),
    }
}
