use rust_lib::{
    config::Config,
    healthchecker,
    logger::{level::Parser, logger_init},
};
use std::sync::{atomic::AtomicBool, Arc};

use bsc_adapter::BinanceHandler;
use db::Database;
use log::{error, info, LevelFilter};
use substrate_api_client::Pair;
use tokio::sync::mpsc;

#[allow(clippy::too_many_lines)]
fn main() {
    // Init logger
    match Parser::from_str(&Config::key_from_value("LOGGER_LEVEL").expect("Missing env: LOGGER_LEVEL")) {
        Some(level) => logger_init(level),
        None => logger_init(LevelFilter::Trace),
    }

    // Read tokio options from env
    let workers_number = Config::key_from_value("WORKERS_NUMBER").expect("Missing env: WORKERS_NUMBER");
    let workers_number = workers_number
        .parse::<usize>()
        .expect("WORKERS_NUMBER env must be decimal number");

    if workers_number < 2 {
        error!("Workers number {} is too less! Must be at least 2!", workers_number);
    }

    let (binance_tx, binance_rx) = mpsc::channel(1024);
    let (bsc_listen_tx, bsc_listen_rx) = mpsc::channel(1024);
    // let (rollback_tx, rollback_rx) = mpsc::channel(1024);
    // let status = Arc::new(AtomicBool::new(true));

    let binance_url = Config::key_from_value("BINANCE_URL").expect("Missing env BINANCE_URL");
    let token_contract_address = Config::key_from_value("ADDRESS_TOKENS").expect("Missing env ADDRESS_TOKENS");
    let nft_contract_address = Config::key_from_value("ADDRESS_NFT").expect("Missing env ADDRESS_NFT");

    // TODO get from vault
    let binance_master_key = "98a946173492e8e5b73577341cea3c3b8e92481bfcea038b8fd7c1940d0cd42f";

    // Read healthchecker options from env file
    let healthchecker_address = Config::key_from_value("HEALTHCHECK").expect("Missing env HEALTHCHECK");

    // Read blockchain connection options from env file
    let url = Config::key_from_value("REALIS_URL").expect("Missing env URL");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers_number)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // Init some variables
        let status = Arc::new(AtomicBool::new(true));

        let db_host = Config::key_from_value("DATABASE_HOST").expect("Missing env DATABASE_HOST");
        let db_port = Config::key_from_value("DATABASE_PORT").expect("Missing env DATABASE_PORT");
        let db_user = Config::key_from_value("DATABASE_USER").expect("Missing env DATABASE_USER");
        let db_password = Config::key_from_value("DATABASE_PASSWORD").expect("Missing env DATABASE_PASSWORD");
        let db_name = Config::key_from_value("DATABASE_NAME").expect("Missing env DATABASE_NAME");

        let db = Arc::new(
            Database::new(&format!(
                "host={} port={} user={} password={} dbname={}",
                db_host, db_port, db_user, db_password, db_name
            ))
            .await
            .unwrap(),
        );
        match db.import_tables_from_file("./db/res/tables.sql").await {
            Ok(_) => info!("Creating tables was successful"),
            Err(error) => error!("Cannot create tables: {:?}", error),
        }

        // Init listener modules

        let mut modules = vec![];

        modules.push(tokio::spawn({
            let status = Arc::clone(&status);
            async move {
                healthchecker::listen(status, &healthchecker_address).await;
            }
        }));

        match Config::key_from_value("RESTORE").map(|value| value == *"true") {
            Ok(true) => {
                match realis_listener::BlockListener::new_with_restore(
                    &url,
                    binance_tx,
                    Arc::clone(&status),
                    db::Database::new(&format!(
                        "host={} port={} user={} password={} dbname={}",
                        db_host, db_port, db_user, db_password, db_name
                    ))
                    .await
                    .unwrap(),
                )
                .await
                {
                    Ok((mut listener, restore)) => {
                        modules.push(tokio::spawn({
                            async move {
                                listener.listen().await;
                            }
                        }));
                        modules.push(tokio::spawn({
                            async move {
                                restore.await;
                            }
                        }));
                    }
                    Err(error) => error!("Fail to restore - {:?}", error),
                }
            }
            Ok(false) | Err(_) => {
                let mut listener =
                    realis_listener::BlockListener::new(&url, binance_tx, Arc::clone(&status)).unwrap();
                modules.push(tokio::spawn({
                    async move {
                        listener.listen().await;
                    }
                }));
            }
        }

        let binance_handler = BinanceHandler::new(
            binance_rx,
            Arc::clone(&status),
            &binance_url,
            token_contract_address,
            nft_contract_address,
            binance_master_key,
        );
        modules.push(tokio::spawn(binance_handler.handle()));

        let bsc_listener = bsc_listener::BlockListener::new(binance_url, bsc_listen_tx, Arc::clone(&status));

        modules.push(tokio::spawn({
            async move {
                bsc_listener.listen().await;
            }
        }));

        let pair = Pair::from_string(
            "fault pretty bird biology budget table symptom build option wrist time detail",
            None,
        )
        .unwrap();

        let realis_adapter = realis_adapter::RealisAdapter::new(bsc_listen_rx, Arc::clone(&status), &url, pair);

        modules.push(tokio::spawn({
            async move {
                realis_adapter.handle().await;
            }
        }));

        for task in modules {
            let _result = task.await;
        }
    });
}
