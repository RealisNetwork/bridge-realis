pub use substrate_api_client::{BlockNumber, Hash};
use web3::types::{H256, U64};

/// Undecoded bsc event
#[derive(Debug, Clone)]
pub struct RawEvent {
    pub block_number: Option<U64>,
    pub hash: H256,
    pub data: Vec<u8>,
}
