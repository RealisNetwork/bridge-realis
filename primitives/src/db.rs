/// This enum is being casted to u32 so order matters, starts from 1.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    // Got from events
    Got,
    // Sent to blockchain, wait for result
    InProgress,
    // Blockchain inBlock
    Success,
    // Blockchain fail
    Error,
    // Rollback success
    Rollbacked,
}
