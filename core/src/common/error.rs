use crate::core::message::error::MessageError;
use crate::core::tunnel::error::TunnelError;

#[derive(Debug)]
pub enum Error {
    MessageError(MessageError),
    TunnelError(TunnelError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MessageError(e) => write!(f, "MessageError: {e}"),
            Error::TunnelError(e) => write!(f, "MessageError: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<TunnelError> for Error {
    fn from(error: TunnelError) -> Self {
        Self::TunnelError(error)
    }
}

impl From<MessageError> for Error {
    fn from(error: MessageError) -> Self {
        Self::MessageError(error)
    }
}
