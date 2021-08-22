pub mod parse;

pub mod contract {
    use std::{str::FromStr, time::Duration};

    use log::{error, info};
    use primitives::Config;
    use tokio::time::sleep;
    use web3::{contract::Contract, transports::WebSocket, types::Address, Web3};

    /// # Panics
    ///
    /// Connect to bsc
    pub async fn connect() -> Web3<WebSocket> {
        let url = Config::key_from_value("URL_BSC");
        // Connect to bsc
        let mut wss = WebSocket::new(&url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!("Cannot connect {:?}", error);
                    info!("Try reconnect");
                    wss = WebSocket::new(&url).await;
                }
            }
            // Wait a bit before reconnect
            sleep(Duration::from_millis(1000)).await;
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
            Address::from_str("0x6d749dD747da1754Ef16B3fA2E779834CF636805")
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
            Address::from_str("0x3eEdAa20Ec4AfF8472355c138F39ca2a4b02e518")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP721.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    pub async fn connect_eth() -> Web3<WebSocket> {
        let url = "wss://mainnet.infura.io/ws/v3/eb804ac058aa4ab38efc538c2153ee9b";
        // Connect to bsc
        let mut wss = WebSocket::new(&url).await;
        loop {
            match wss {
                Ok(_) => break,
                Err(error) => {
                    error!("Cannot connect {:?}", error);
                    info!("Try reconnect");
                    wss = WebSocket::new(&url).await;
                }
            }
            // Wait a bit before reconnect
            sleep(Duration::from_millis(1000)).await;
        }

        web3::Web3::new(wss.unwrap())
    }
}

pub mod accounts {
    // use sp_core::Pair;
    use std::{fs, path::Path, str::FromStr};

    use secp256k1::SecretKey;

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


