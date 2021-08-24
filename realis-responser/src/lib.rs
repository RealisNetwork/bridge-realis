use primitives::{Config, ResponderRequest};

use log::{debug, error, info};
use ratsio::{StanClient, StanOptions};
use serde_json::{json, Value};

/// # Panics
pub async fn listen(receiver: ResponderRequest) {
    let client_id = Config::key_from_value("RESPONDER_ID");
    let opts = StanOptions::with_options(
        Config::key_from_value("NATS_OPT"),
        Config::key_from_value("CLUSTER_ID"),
        client_id[..].parse().unwrap(),
    );

    let subject = Config::key_from_value("RESPONDER_SUBJECT");

    match StanClient::from_options(opts).await {
        Ok(stan_client) => match Some(receiver.clone()) {
            None => {}
            Some(response) => {
                let value = parse(response);
                let json = value.to_string();
                match stan_client.publish(subject.clone(), json.as_bytes()).await {
                    Ok(_) => info!("Response sent: {:?}!", json),
                    Err(error) => error!("{:?}", error),
                }
            }
        },
        Err(error) => error!("{:?}", error),
    }
}

fn parse(response: ResponderRequest) -> Value {
    debug!("Catch Response!");
    match response {
        ResponderRequest::TransferTokenToBSC(raw_request) => {
            json!({
                "result": {
                    "request": raw_request,
                    "response": {
                        "type": "Right",
                        "value": {
                            "tx_id": "some tx_id",
                            "token_id": "some token_id"
                        }
                    }
                }
            })
        }
        ResponderRequest::TransferNftToBSC(raw_request) => {
            json!({
                "result": {
                    "request": raw_request,
                    "response": {
                        "type": "Right",
                        "value": {
                            "tx_id": "some tx_id",
                            "token_id": "some token_id"
                        }
                    }
                }
            })
        }
        ResponderRequest::TransferTokenToRealis(raw_request) => {
            json!({
                "result": {
                    "request": raw_request,
                    "response": {
                        "type": "Right",
                        "value": {
                            "tx_id": "some tx_id",
                            "token_id": "some token_id"
                        }
                    }
                }
            })
        }
        ResponderRequest::TransferNftToRealis(raw_request) => {
            json!({
                "result": {
                    "request": raw_request,
                    "response": {
                        "type": "Right",
                        "value": {
                            "tx_id": "some tx_id",
                            "token_id": "some token_id"
                        }
                    }
                }
            })
        }
        ResponderRequest::Error(raw_request) => json!({
            "version": "version",
            "lang": "some lang",
            "method": "error",
            "res": {
                "req": raw_request,
                "result": 100,
                "status": 0
            }
        }),
    }
}
