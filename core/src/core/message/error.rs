#[derive(Debug, Copy, Clone)]
pub enum MessageError {
    MessageEmpty,
    MessageTooLong,
    InvalidType,
    InvalidString,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::MessageEmpty => write!(f, "message cannot be empty"),
            MessageError::MessageTooLong => write!(f, "message length exceeded limit"),
            MessageError::InvalidType => write!(f, "invalid message type"),
            MessageError::InvalidString => write!(f, "invalid message string"),
        }
    }
}

impl std::error::Error for MessageError {}
