use primitives::{Config, ResponderRequest};

use log::{debug, error, info};
use ratsio::{StanClient, StanOptions};
use serde_json::{json, Value};

/// # Panics
pub async fn listen(receiver: ResponderRequest) {
    let client_id = Config::key_from_value("RESPONDER_ID").unwrap();
    let opts = StanOptions::with_options(
        Config::key_from_value("NATS_OPT").unwrap(),
        Config::key_from_value("CLUSTER_ID").unwrap(),
        client_id[..].parse().unwrap(),
    );

    let subject = Config::key_from_value("RESPONDER_SUBJECT").unwrap();

    match StanClient::from_options(opts).await {
        Ok(stan_client) => match Some(receiver.clone()) {
            None => {}
            Some(response) => {
                debug!("Response Here!");
                let value = parse(response);
                let json = value.to_string();
                match stan_client.publish(subject.clone(), json.as_bytes()).await
                {
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
                "version": raw_request.version,
                "method": "transfer_token_to_bsc",
                "res": {
                   "req": {
                        "bsc_account": raw_request.params.bsc_account,
                        "amount": raw_request.params.amount.to_string()
                    },
                   "result": 100,
                   "status": 0
                },
                "lang": raw_request.lang,
                "id": raw_request.id
            })
        }
        ResponderRequest::TransferNftToBSC(raw_request) => {
            json!({
                "version": raw_request.version,
                "method": "transfer_nft_to_bsc",
                "res": {
                   "req": {
                        "bsc_account": raw_request.params.bsc_account,
                        "amount": raw_request.params.token_id
                    },
                   "result": 100,
                   "status": 0
                },
                "lang": raw_request.lang,
                "id": raw_request.id
            })
        }
        ResponderRequest::TransferTokenToRealis(raw_request) => {
            json!({
                "version": raw_request.version,
                "method": "transfer_token_to_realis",
                "res": {
                   "req": {
                        "account_id": raw_request.params.account_id,
                        "amount": raw_request.params.amount
                    },
                   "result": 100,
                   "status": 0
                },
                "lang": raw_request.lang,
                "id": raw_request.id
            })
        }
        ResponderRequest::TransferNftToRealis(raw_request) => {
            json!({
                "version": raw_request.version,
                "method": "transfer_nft_to_realis",
                "res": {
                   "req": {
                        "account_id": raw_request.params.account_id,
                        "amount": raw_request.params.token_id
                    },
                   "result": 100,
                   "status": 0
                },
                "lang": raw_request.lang,
                "id": raw_request.id
            })
        }
        ResponderRequest::Error() => json!({
            "version": "raw_request.version",
            "lang": "some lamg",
            "method": "error",
            "res": {
                "result": 100,
                "status": 0
            }
        }),
    }
}
