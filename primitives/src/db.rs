/// This enum is being casted to u32 so order matters, starts from 1.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    // Got from message-broker
    Got,
    // Sent to blockchain , wait for result
    InProgress,
    // In block
    Success,
    // Blockchain fail
    Error,
}
