pub mod contract {
    use log::{error, info};
    use std::str::FromStr;

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
        }

        web3::Web3::new(wss.unwrap())
    }

    /// # Panics
    ///
    /// Create new token contract
    pub async fn token_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        let address: Address =
            Address::from_str("0x987893D34052C07F5959d7e200E9e10fdAf544Ef")
                .unwrap();

        let json_abi = include_bytes!("./../res/BEP20.abi");

        Contract::from_json(web3.eth(), address, json_abi).unwrap()
    }

    /// # Panics
    ///
    /// Create new nft contract
    pub async fn nft_new(url: &str) -> Contract<WebSocket> {
        let web3 = connect(url).await;
        let address: Address =
            Address::from_str("0x2a5252DA791289485919687c1faD2a6d60311f25")
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
