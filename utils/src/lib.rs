pub mod logger {
    use slog::{o, Drain};

    #[must_use]
    pub fn new() -> slog::Logger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, o!())
    }
}

pub mod contract {
    use std::str::FromStr;

    use web3::{
        contract::Contract,
        transports::WebSocket,
        types::Address,
        Web3
    };

    use slog::{error, info};
    use crate::logger;

    async fn connect(url: &str) -> Web3<WebSocket> {
        let log = logger::new();
        // Connect to bsc
        let mut wss = WebSocket::new(url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!(log, "Cannot connect {:?}", error);
                    info!(log, "Try reconnect");
                    wss = WebSocket::new(url).await;
                }
            }
        }

        web3::Web3::new(wss.unwrap())
    }

    pub async fn token_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        let address: Address =
            Address::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    pub async fn nft_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        let address: Address =
            Address::from_str("0x8A19360f2EC953b433D92571120bb5ef755b3d17")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP721.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }
}

pub mod accounts {
    use std::{fs, path::Path, str::FromStr};
    use secp256k1::SecretKey;

    pub fn realis<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(&path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }
}
