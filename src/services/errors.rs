#[derive(Debug)]
pub enum RpcError {
    Api,
    BlockNotFound,
    EventsNotFound,
}
