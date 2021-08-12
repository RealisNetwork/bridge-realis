use bsc_sender::BscSender;
use futures::{Stream, StreamExt};
use log::info;
use ratsio::{RatsioError, StanClient, StanMessage, StanOptions, StanSid};
use realis_sender::RealisSender;
use runtime::AccountId;
use sp_core::{crypto::Ss58Codec, H160, U256};
use std::str::FromStr;
use web3::contract::deploy::Error;

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
pub async fn message_broker() {
    logger_setup();
    let mut stan_client = sub_stan().await.unwrap();
    while let Some(message) = stan_client.1.next().await {
        info!(
            " << 1 >> got stan message --- {:?}\n\t{:?}",
            &message,
            String::from_utf8_lossy(message.payload.as_ref())
        );
        let message_string = String::from_utf8_lossy(message.payload.as_ref());
        let accounts: Vec<&str> = message_string.split(' ').collect();
        if accounts[7].starts_with("Value") {
            listen_token(accounts).await.unwrap();
        } else if accounts[7].starts_with("TokenId") {
            listen_nft(accounts).await.unwrap();
        }
    }
}

async fn listen_token(accounts: Vec<&str>) -> Result<(), Error> {
    logger_setup();
    if accounts[2].starts_with("0x") {
        let account_id_bsc = H160::from_str(&accounts[2]).unwrap();
        let account_id_realis = AccountId::from_ss58check(&accounts[5]).unwrap();
        let amount = u128::from_str(&accounts[8]).unwrap();
        let value = accounts.clone();
        RealisSender::send_token_to_realis(
            account_id_bsc,
            account_id_realis.clone(),
            amount,
        )
        .await;
        println!("{:?}", account_id_bsc);
        println!("{:?}", account_id_realis);
        println!("{:?}", value);
    } else {
        let account_id_realis = AccountId::from_ss58check(&accounts[2]).unwrap();
        let account_id_bsc = H160::from_str(&accounts[5]).unwrap();
        let value = u128::from_str(&accounts[8]).unwrap();
        BscSender::send_token_to_bsc(
            account_id_realis.clone(),
            account_id_bsc,
            value,
        )
        .await;
        println!("{:?}", account_id_realis);
        println!("{:?}", account_id_bsc);
        println!("{:?}", value);
    }
    Ok(())
}

async fn listen_nft(accounts: Vec<&str>) -> Result<(), Error> {
    logger_setup();
    if accounts[2].starts_with("0x") {
        let account_id_bsc = H160::from_str(&accounts[2]).unwrap();
        let account_id_realis = AccountId::from_ss58check(&accounts[5]).unwrap();
        let amount = U256::from_dec_str(accounts[8]).unwrap();
        let token_type: u8 = accounts[11].parse().unwrap();
        let rarity = accounts[14].parse().unwrap();
        RealisSender::send_nft_to_realis(
            account_id_bsc,
            account_id_realis.clone(),
            amount,
            token_type,
            rarity,
        )
        .await;
        println!("{:?}", account_id_bsc);
        println!("{:?}", account_id_realis);
        println!("{:?}", amount);
    } else {
        let account_id_realis = AccountId::from_ss58check(&accounts[2]).unwrap();
        let account_id_bsc = H160::from_str(&accounts[5]).unwrap();
        let value = web3::types::U256::from_dec_str(accounts[8]).unwrap();
        let token_type: u8 = accounts[11].parse().unwrap();
        BscSender::send_nft_to_bsc(
            account_id_realis.clone(),
            account_id_bsc,
            value,
            token_type,
        )
        .await;
        println!("{:?}", account_id_realis);
        println!("{:?}", account_id_bsc);
        println!("{:?}", value);
    }
    Ok(())
}

async fn sub_stan(
) -> Result<(StanSid, impl Stream<Item = StanMessage> + Send + Sync), RatsioError>
{
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
    let (sid, subscription) = stan_client
        .subscribe("realis-bridge", None, None)
        .await
        .unwrap();
    Ok((sid, subscription))
}
