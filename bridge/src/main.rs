use rust_lib::{config::Config, healthchecker, logger::{logger_init, level::Parser}};
use std::sync::{atomic::AtomicBool, Arc};

use log::{error, info, LevelFilter};
use tokio::sync::mpsc;

#[allow(clippy::too_many_lines)]
fn main() {
    // Init logger
    match Parser::from_str(&Config::key_from_value("LOGGER_LEVEL")
        .expect("Missing env: LOGGER_LEVEL")) {
        Some(level) => logger_init(level),
        None => logger_init(LevelFilter::Trace)
    }

    // Read tokio options from env
    let workers_number =
        Config::key_from_value("WORKERS_NUMBER").expect("Missing env: WORKERS_NUMBER");
    let workers_number = workers_number
        .parse::<usize>()
        .expect("WORKERS_NUMBER env must be decimal number");

    if workers_number < 2 {
        error!(
            "Workers number {} is too less! Must be at least 2!",
            workers_number
        );
    }

    // Read db options from env file
    let db_host = Config::key_from_value("DATABASE_HOST").expect("Missing env DATABASE_HOST");
    let db_port = Config::key_from_value("DATABASE_PORT").expect("Missing env DATABASE_PORT");
    let db_user = Config::key_from_value("DATABASE_USER").expect("Missing env DATABASE_USER");
    let db_password = Config::key_from_value("DATABASE_PASSWORD").expect("Missing env DB_PASSWORD");
    let db_name = Config::key_from_value("DATABASE_NAME").expect("Missing env DATABASE_NAME");
    let db_params = format!(
        "host={} port={} user={} password={} dbname={}",
        db_host, db_port, db_user, db_password, db_name
    );

    // Read healthchecker options from env file
    let healthchecker_address =
        Config::key_from_value("HEALTHCHECK").expect("Missing env HEALTHCHECK");

    // Read blockchain connection options from env file
    let url = Config::key_from_value("URL").expect("Missing env URL");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers_number)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // Init some variables
        let (handler_tx, handler_rx) = mpsc::channel(32);
        let status = Arc::new(AtomicBool::new(true));

        // Init database and import tables
        let db = db::Database::new(&db_params).await.unwrap();
        match db.import_tables_from_file("./db/res/tables.sql").await {
            Ok(()) => info!("Creating tables was successful!"),
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
                match adapter::BlockListener::new_with_restore(
                    &url,
                    handler_tx,
                    db::Database::new(&db_params).await.unwrap(),
                    Arc::clone(&status),
                )
                    .await
                {
                    Ok((mut adapter, restore)) => {
                        modules.push(tokio::spawn({
                            async move {
                                adapter.listen().await;
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
                    adapter::BlockListener::new(&url, handler_tx, Arc::clone(&status))
                        .unwrap();
                modules.push(tokio::spawn({
                    async move {
                        adapter.listen().await;
                    }
                }));
            }
        }

        for task in modules {
            let _result = task.await;
        }
    });
}