use log::info;
use futures::StreamExt;
use ratsio::{RatsioError, StanClient, StanOptions};
use runtime::AccountId;
use sp_core::{crypto::Ss58Codec, H160};
use web3::types::Address;
use std::str::FromStr;
use realis_sender::RealisSender;
use bsc_sender::BscSender;

pub fn logger_setup() {
    use log::LevelFilter;
    use std::io::Write;
    use env_logger::Builder;

    let _ = Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "[{}] - {}",
                     record.level(),
                     record.args()
            )
        })
        .filter(None, LevelFilter::Trace)
        .try_init();
}


#[tokio::main]
async fn main() -> Result<(), RatsioError> {
    use sp_core::crypto::Ss58Codec;
    logger_setup();
    // Create stan options
    let client_id = "realis-bridge".to_string();
    let opts = StanOptions::with_options("localhost:4222", "test-cluster", &client_id[..]);
    //Create STAN client
    let stan_client = StanClient::from_options(opts).await?;

    //Subscribe to STAN subject 'foo'
    let (sid, mut subscription) = stan_client.subscribe("realis-bridge", None, None).await?;
    let thread = tokio::spawn(async move {
        while let Some(message) = subscription.next().await {
            info!(" << 1 >> got stan message --- {:?}\n\t{:?}", &message,
                String::from_utf8_lossy(message.payload.as_ref()));
            let message_string = &*String::from_utf8_lossy(message.payload.as_ref());
            let accounts: Vec<&str> = message_string.split(' ').collect();
            let account_id_realis = AccountId::from_ss58check(accounts[2]).unwrap();
            let account_id_bsc = H160::from_str(accounts[5]).unwrap();
            let value = u128::from_str(accounts[8]).unwrap();
            match accounts[2] {
                "5" => BscSender::send_token_to_bsc(account_id_realis.clone(), account_id_bsc, value)
                    .await,
                // Some("0x") => ,
                _ => {}
            }
            println!("{:?}", account_id_realis);
            println!("{:?}", account_id_bsc);
            println!("{:?}", value);
        }
        info!(" ----- the subscription loop is done ---- ")
    });

    //Publish some mesesages to 'foo', use 'cargo run --example stan_publish foo "hi there"'
    thread.await.unwrap();
    Ok(())
}