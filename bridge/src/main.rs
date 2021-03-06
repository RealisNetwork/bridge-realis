use rust_lib::{async_logger, config::Config};
use std::sync::Arc;

use bsc_adapter::BinanceHandler;
use db::Database;
use futures::future::join_all;
use log::{error, info, LevelFilter};
use realis_listener::listener_builder::BlockListenerBuilder;
use rust_lib::{blockchain::wallets::RealisWallet, healthchecker::HealthChecker};
use tokio::sync::mpsc;

#[allow(clippy::too_many_lines)]
fn main() {
    // Init logger
    let (_, _guard) = match Config::key_from_value("LOGGER_LEVEL") {
        Ok(level) => async_logger::init(level),
        Err(_) => async_logger::init(LevelFilter::Trace.to_string()),
    };

    // Read tokio options from env
    let workers_number = Config::key_from_value("WORKERS_NUMBER").expect("Missing env: WORKERS_NUMBER");
    let workers_number = workers_number
        .parse::<usize>()
        .expect("WORKERS_NUMBER env must be decimal number");

    if workers_number < 2 {
        error!("Workers number {} is too less! Must be at least 2!", workers_number);
    }

    let binance_url = Config::key_from_value("BINANCE_URL").expect("Missing env BINANCE_URL");
    let token_contract_address = Config::key_from_value("ADDRESS_TOKENS").expect("Missing env ADDRESS_TOKENS");
    let nft_contract_address = Config::key_from_value("ADDRESS_NFT").expect("Missing env ADDRESS_NFT");
    let token_topic = Config::key_from_value("TOKEN_TOPIC").expect("Missing env TOKEN_TOPIC");
    let nft_topic = Config::key_from_value("NFT_TOPIC").expect("Missing env NFT_TOPIC");

    // Read healthchecker options from env file
    let healthchecker_address = Config::key_from_value("HEALTHCHECK").expect("Missing env HEALTHCHECK");

    // Read blockchain connection options from env file
    let url = Config::key_from_value("REALIS_URL").expect("Missing env URL");

    let db_host = Config::key_from_value("DATABASE_HOST").expect("Missing env DATABASE_HOST");
    let db_port = Config::key_from_value("DATABASE_PORT").expect("Missing env DATABASE_PORT");
    let db_user = Config::key_from_value("DATABASE_USER").expect("Missing env DATABASE_USER");
    let db_password = Config::key_from_value("DATABASE_PASSWORD").expect("Missing env DATABASE_PASSWORD");
    let db_name = Config::key_from_value("DATABASE_NAME").expect("Missing env DATABASE_NAME");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers_number)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // Init some variables
        let (binance_tx, binance_rx) = mpsc::channel(1024);
        let (realis_tx, realis_rx) = mpsc::channel(1024);
        let pair = rust_lib::blockchain::wallets::BridgeMaster::get_private();
        // TODO get from vault
        let binance_master_key = "98a946173492e8e5b73577341cea3c3b8e92481bfcea038b8fd7c1940d0cd42f";

        let health_checker = HealthChecker::new(&healthchecker_address, 10000)
            .await
            .expect("Healthchecker error");

        let db = Arc::new(
            Database::new(
                &db_host,
                &db_port,
                &db_user,
                &db_password,
                &db_name,
                true,
                health_checker.clone(),
            )
            .await
            .unwrap(),
        );
        match db.import_tables_from_file("./db/res/tables.sql").await {
            Ok(_) => info!("Creating tables was successful"),
            Err(error) => error!("Cannot create tables: {:?}", error),
        }

        // Init listener modules

        let mut modules = vec![];

        let binance_handler = BinanceHandler::new(
            binance_rx,
            realis_tx.clone(),
            health_checker.clone(),
            &binance_url,
            token_contract_address.clone(),
            nft_contract_address.clone(),
            binance_master_key,
            Arc::clone(&db),
        );
        modules.push(tokio::spawn(binance_handler.handle()));

        let realis_adapter = realis_adapter::RealisAdapter::new(
            realis_rx,
            binance_tx.clone(),
            health_checker.clone(),
            &url,
            pair,
            Arc::clone(&db),
        );

        modules.push(tokio::spawn({
            async move {
                realis_adapter.handle().await;
            }
        }));

        match Config::key_from_value("RESTORE").map(|value| value == *"true") {
            Ok(true) => {
                let last_block = db.get_last_block_realis().await.unwrap_or(0);
                let (mut listener, tx) =
                    BlockListenerBuilder::new(&url, binance_tx, health_checker.clone(), Arc::clone(&db)).build();
                modules.push(tokio::spawn({
                    async move {
                        listener.listen_with_restore(last_block, tx).await;
                    }
                }));
            }
            Ok(false) | Err(_) => {
                let (mut listener, _) =
                    BlockListenerBuilder::new(&url, binance_tx, health_checker.clone(), Arc::clone(&db)).build();
                modules.push(tokio::spawn({
                    async move {
                        listener.listen().await;
                    }
                }));
            }
        }

        match Config::key_from_value("RESTORE").map(|value| value == *"true") {
            Ok(true) => {
                let last_block = db.get_last_block_bsc().await.unwrap();
                let mut bsc_listener = bsc_listener::BlockListener::new(
                    binance_url,
                    realis_tx,
                    health_checker.clone(),
                    Arc::clone(&db),
                    &token_contract_address,
                    &nft_contract_address,
                    &token_topic,
                    &nft_topic,
                )
                .await
                .unwrap();
                modules.push(tokio::spawn({
                    async move {
                        bsc_listener.listen_with_restore(last_block).await;
                    }
                }));
            }
            Ok(false) | Err(_) => {
                let mut bsc_listener = bsc_listener::BlockListener::new(
                    binance_url,
                    realis_tx,
                    health_checker.clone(),
                    Arc::clone(&db),
                    &token_contract_address,
                    &nft_contract_address,
                    &token_topic,
                    &nft_topic,
                )
                .await
                .unwrap();
                modules.push(tokio::spawn({
                    async move {
                        bsc_listener.listen().await;
                    }
                }));
            }
        }

        join_all(modules).await;
    });
}
