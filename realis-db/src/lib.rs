use bip39::{Language, Mnemonic, MnemonicType};
use log::{error, info, trace, warn};
use postgres::NoTls;
use primitives::{Config, Raw, Request, ResponderRequest, Error};
use sp_core::{
    crypto::{AccountId32, Ss58Codec},
    Pair,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_postgres::Client;
use web3::api::{ParityAccounts, Namespace};
use utils::*;
use utils::contract::{connect, connect_eth};
use web3::types::Address;

pub async fn listen(receiver: Receiver<Request>, sender: Sender<Request>) {
    let mut receiver = receiver;

    let client_result = connect_to_db().await;

    match client_result {
        Ok(client) => loop {
            match receiver.recv().await {
                None => break,
                Some(request) => {
                    info!("Realis db - listen - Got request: {:?}", &request);
                    if let Some(request) = process_request(&client, &request).await {
                        let result = sender.send(request.clone()).await;
                        match result {
                            Ok(_) => {
                                info!("Realis db - listen - send: {:?}", request)
                            }
                            Err(error) => error!(
                                "Realis db - listen -\
                                 unable to send response: {:?}",
                                error
                            ),
                        }
                    }
                }
            }
        },
        Err(error) => {
            error!("Cannot connect to database: {:?}", error)
        }
    }
    warn!("Realis db - listen - channel closed!");
}

async fn insert_db_wallet(
    client: &Client,
    user_id: String,
    account_id: AccountId32,
    mnemonic: String,
) -> Result<u64, tokio_postgres::Error> {
    println!("Connect!");
    let account_id_str: String = account_id.to_string();
    println!("Preparing query...");
    let query = client
        .execute_raw(
            "INSERT INTO wallet_to_user \
            (user_id, account_id, mnemonic) VALUES ($1, $2, $3)",
            &[&user_id, &account_id_str, &mnemonic],
        )
        .await;
    println!("All done!");
    query
}

#[tokio::main]
async fn main() {
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    let phrase = mnemonic.phrase();

    println!("phrase: {}", phrase);

    let connect = connect_eth().await;

    let transport = ParityAccounts::new(connect);

    let address =
        transport.parity_new_account_from_phrase(phrase, "").await;
    // println!("account_id: {:?}", pair.to_string());
    println!("account_id: {:?}", address);

}

async fn connect_to_db() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect(
        &*Config::key_from_value("DATABASE_CFG"),
        NoTls,
    )
        .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    println!("Connect to database!");
    Ok(client)
}

async fn get_account_id_by_user_id(
    client: &Client,
    user_id: String,
) -> Result<String, Error> {
    match client
        .query_one(
            "SELECT account_id FROM wallet_to_user WHERE user_id=$1",
            &[&user_id],
        )
        .await
    {
        Ok(row) => {
            let column_account_id: String = row.get(0);
            Ok(column_account_id)
        }
        Err(_) => Err(Error::UserNotFound(user_id)),
    }
}

async fn process_request(client: &Client, request: &Request) {
   println!("Ok!");
}
