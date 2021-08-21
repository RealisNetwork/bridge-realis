use futures::{Stream, StreamExt};
use log::{error, info, trace};
use primitives::{Config, Request, ResponderRequest};
use ratsio::{StanClient, StanMessage, StanOptions};

use bsc_sender::BscSender;
use realis_sender::RealisSender;

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
pub async fn listen() {
    logger_setup();
    info!("Start connection to nats streaming! 123");
    let mut subscription = sub_stan().await;
    info!("Connect!");
    while let Some(message) = subscription.next().await {
        info!("Got message");
        match parse(&message) {
            Ok(request) => match request {
                Request::TransferTokenToBSC(raw_request) => {
                    info!("Message {:?}", raw_request);
                    BscSender::send_token_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.amount,
                    )
                    .await;
                    trace!("Drop Here");
                    realis_responser::listen(
                        ResponderRequest::TransferTokenToBSC(raw_request.clone()),
                    )
                    .await;
                }
                Request::TransferNftToBSC(raw_request) => {
                    info!("Message {:?}", raw_request);
                    BscSender::send_nft_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                        raw_request.params.rarity.parse().unwrap(),
                    )
                    .await;
                    realis_responser::listen(ResponderRequest::TransferNftToBSC(
                        raw_request.clone(),
                    ))
                    .await;
                }
                Request::TransferTokenToRealis(raw_request) => {
                    info!("Message {:?}", raw_request);
                    RealisSender::send_token_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.amount * 10_000_000_000,
                    );
                    realis_responser::listen(
                        ResponderRequest::TransferTokenToRealis(
                            raw_request.clone(),
                        ),
                    )
                    .await;
                }
                Request::TransferNftToRealis(raw_request) => {
                    info!("Message {:?}", raw_request);
                    RealisSender::send_nft_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                        raw_request.params.rarity.parse().unwrap(),
                    );
                    realis_responser::listen(
                        ResponderRequest::TransferNftToRealis(
                            raw_request.clone(),
                        ),
                    );
                }
                Request::WithdrawFromBSC(raw_request) => {
                    info!("Message {:?}", raw_request);
                    RealisSender::send_token_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.amount * 10_000_000_000,
                    );
                }
                Request::WithdrawFromRealis(raw_request) => {
                    info!("Message {:?}", raw_request);
                    BscSender::send_token_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.amount,
                    )
                    .await;
                }
            },
            Err(error) => error!("{:?}", error),
        }
    }
}

async fn sub_stan() -> impl Stream<Item = StanMessage> {
    // Create stan options
    info!("Start connect to NATS_Streaming!");
    let client_id = Config::key_from_value("CLIENT_ID").unwrap();
    let opts = StanOptions::with_options(
        Config::key_from_value("NATS_OPT").unwrap(),
        Config::key_from_value("CLUSTER_ID").unwrap(),
        String::from(&client_id[..]),
    );
    // Create STAN client
    let stan_client = StanClient::from_options(opts).await.unwrap();

    // Subscribe to STAN subject 'realis-bridge'
    stan_client
        .subscribe(Config::key_from_value("SUBJECT").unwrap(), None, None)
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
