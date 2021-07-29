pub mod contract {
    use log::{error, info};
    use std::str::FromStr;
    use tokio::time::{sleep, Duration};

    use web3::{
        contract::Contract, transports::WebSocket, types::Address, Web3,
    };

    async fn connect(url: &str) -> Web3<WebSocket> {
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
            sleep(Duration::from_millis(1000));
        }

        web3::Web3::new(wss.unwrap())
    }

    /// # Panics
    ///
    /// Create new token contract
    pub async fn token_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        // TODO get from config file
        let address: Address =
            Address::from_str("0x30a02a714Ea7674F1988ED5d81094F775b28E611")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    /// # Panics
    ///
    /// Create new nft contract
    pub async fn nft_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        // TODO get from config file
        let address: Address =
            Address::from_str("0xeabfdb7ab0774d2f887e99f87e9279a6ee5c1431")
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
