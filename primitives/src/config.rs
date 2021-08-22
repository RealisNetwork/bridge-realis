use crate::Error;

pub struct Config {
    pub query: String,
    pub filename: String,
    pub database_cfg: String,
    pub url: String,
    pub nats_options: String,
    pub cluster_id: String,
    pub client_id: String,
    pub subject: String,
}

impl Config {
    /// # Errors
    ///
    /// If `.env` file can not be parsed
    ///
    /// # Panics
    pub fn key_from_value(key: &str) -> String {
        let dotenv = dotenv::dotenv().ok();
        let value = dotenv::var(key).unwrap();
        value
    }

    #[must_use]
    pub fn is_restore() -> bool {
        dotenv::dotenv().ok();

        match dotenv::var("IS_RESTORE") {
            Ok(value) => matches!(&value.to_lowercase()[..], "true"),
            Err(_) => false,
        }
    }
}
