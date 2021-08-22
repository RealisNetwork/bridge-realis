use futures::{Stream, StreamExt};
use log::{error, info, trace};
use primitives::{Config, Request, ResponderRequest, External, Error};
use ratsio::{StanClient, StanMessage, StanOptions};

use bsc_sender::BscSender;
use realis_sender::RealisSender;
use primitives::Internal;
use primitives::Connections::NatsSend;

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
                    let sender = BscSender::send_token_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.amount,
                    )
                    .await;
                    // match sender {
                    // Ok(sender) => realis_responser::listen(
                    //     ResponderRequest::TransferTokenToBSC(raw_request.clone()),
                    // )
                    // .await,
                    //     Err(error) => {
                    //         error!("Error when triyng send transaction: {:?}", error);
                    //         realis_responser::listen(
                    //             ResponderRequest::Error(Error::CannotSendExtrinsicRealis),
                    //         )
                    //             .await
                    //     }
                    // }
                }
                Request::TransferNftToBSC(raw_request) => {
                    info!("Message {:?}", raw_request);
                    let sender = BscSender::send_nft_to_bsc(
                        raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.bsc_account.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                        raw_request.params.rarity.parse().unwrap(),
                    )
                    .await;
                    // match sender {
                    //     Ok(sender) => realis_responser::listen(
                    //         ResponderRequest::TransferNftToBSC(raw_request.clone()),
                    //     )
                    //         .await,
                    //     Err(error) => {
                    //         error!("Error when trying send transaction: {:?}", error);
                    //         realis_responser::listen(
                    //             ResponderRequest::Error(Error::CannotSendExtrinsicBSC),
                    //         )
                    //             .await;
                    //     }
                    // }
                }
                Request::TransferTokenToRealis(raw_request) => {
                    info!("Message {:?}", raw_request);
                    let sender = RealisSender::send_token_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.amount * 10_000_000_000,
                    );
                    match sender {
                        Ok(sender) => realis_responser::listen(
                            ResponderRequest::TransferTokenToRealis(raw_request.clone()),
                        )
                            .await,
                        Err(error) => {
                            error!("Error when triyng send transaction: {:?}", error);
                            realis_responser::listen(
                                ResponderRequest::Error(Error::CannotSendExtrinsicRealis),
                            )
                                .await
                        }
                    }
                }
                Request::TransferNftToRealis(raw_request) => {
                    info!("Message {:?}", raw_request);
                    let sender = RealisSender::send_nft_to_realis(
                        raw_request.params.bsc_account.parse().unwrap(),
                        &raw_request.params.account_id.parse().unwrap(),
                        raw_request.params.token_id,
                        raw_request.params.token_type,
                        raw_request.params.rarity.parse().unwrap(),
                    );
                    match sender {
                        Ok(sender) => realis_responser::listen(
                            ResponderRequest::TransferNftToRealis(raw_request.clone()),
                        )
                            .await,
                        Err(error) => {
                            error!("Error when triyng send transaction: {:?}", error);
                            realis_responser::listen(
                                ResponderRequest::Error(Error::CannotSendExtrinsicRealis),
                            )
                                .await
                        }
                    }
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
    let client_id = Config::key_from_value("CLIENT_ID");
    let opts = StanOptions::with_options(
        Config::key_from_value("NATS_OPT"),
        Config::key_from_value("CLUSTER_ID"),
        String::from(&client_id[..]),
    );
    // Create STAN client
    let stan_client = StanClient::from_options(opts).await.unwrap();

    // Subscribe to STAN subject 'realis-bridge'
    stan_client
        .subscribe(Config::key_from_value("SUBJECT"), None, None)
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
    let raw_request =
        serde_json::from_str(&message_string);
    match raw_request {
        Ok(raw_request) => Ok(raw_request),
        Err(raw_error) => Err(raw_error),
    }
}