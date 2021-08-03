pub mod contract {
    use log::{error, info};
    use std::str::FromStr;
    use tokio::time::{delay_for, Duration};

    use web3::{contract::Contract, transports::WebSocket, types::Address, Web3};

    async fn connect() -> Web3<WebSocket> {
        let url = "wss://data-seed-prebsc-1-s1.binance.org:8545/";
        // Connect to bsc
        let mut wss = WebSocket::new(url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!("Cannot connect {:?}", error);
                    info!("Try reconnect");
                    wss = WebSocket::new(url).await;
                }
            }
            // Wait a bit before reconnect
            delay_for(Duration::from_millis(1000)).await;
        }

        web3::Web3::new(wss.unwrap())
    }

    /// # Panics
    ///
    /// Create new token contract
    pub async fn token_new() -> Contract<WebSocket> {
        let web3 = connect().await;
        // TODO get from config file
        let address: Address =
            Address::from_str("0xFF1643d2bdb75F8FB7A22E403E3C88aC3A4A183F")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    /// # Panics
    ///
    /// Create new nft contract
    pub async fn nft_new() -> Contract<WebSocket> {
        let web3 = connect().await;
        // TODO get from config file
        let address: Address =
            Address::from_str("0x57F890bDF1c3622d23Cd81A6133EdCE3128f6dCD")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP721.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }
}

pub mod accounts {
    use secp256k1::SecretKey;
    // use sp_core::Pair;
    use std::{fs, path::Path, str::FromStr};

    /// # Panics
    ///
    /// Read private key from file
    pub fn bsc<P: AsRef<Path>>(path: P) -> SecretKey {
        let string = fs::read_to_string(&path).unwrap();
        SecretKey::from_str(&string).unwrap()
    }

    // pub fn realis<P: AsRef<Path>>(path: P, password: Option<&str>) -> Pair<>
    // {     Pair::from_string(
    //         from_path_to_account(path),
    //         password,
    //     ).unwrap()
    // }
}
