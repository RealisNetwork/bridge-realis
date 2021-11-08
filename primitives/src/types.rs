use sp_core::{H160, H256};
pub use substrate_api_client::BlockNumber;
use web3::types::{self, U64};

pub type BscAccount = H160;
pub type UserId = String;
pub type Amount = u128;
pub type Hash = H256;

/// Undecoded bsc event
#[derive(Debug, Clone)]
pub struct RawEvent {
    pub block_number: Option<U64>,
    pub hash: types::H256,
    pub data: Vec<u8>,
}
