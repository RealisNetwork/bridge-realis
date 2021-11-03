use primitives::Error;

use web3::{contract::Contract, transports::WebSocket, types::Address, Web3};

use std::str::FromStr;

pub struct ConnectionBuilder {
    url: String,
}

impl ConnectionBuilder {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string() }
    }

    pub async fn connect(&self) -> Result<Web3<WebSocket>, Error> {
        Ok(Web3::new(WebSocket::new(&self.url).await.map_err(Error::Web3)?))
    }

    pub async fn token(connection: Web3<WebSocket>, contract_address: &str) -> Result<Contract<WebSocket>, Error> {
        let address =
            Address::from_str(contract_address).map_err(|error| Error::Custom(format!("{:?}", error)))?;

        let abi = include_bytes!("./../res/BEP20.abi");

        Contract::from_json(connection.eth(), address, abi).map_err(|error| Error::Custom(format!("{:?}", error)))
    }

    pub async fn nft(connection: Web3<WebSocket>, contract_address: &str) -> Result<Contract<WebSocket>, Error> {
        let address =
            Address::from_str(contract_address).map_err(|error| Error::Custom(format!("{:?}", error)))?;

        let abi = include_bytes!("./../res/BEP721.abi");

        Contract::from_json(connection.eth(), address, abi).map_err(|error| Error::Custom(format!("{:?}", error)))
    }
}
