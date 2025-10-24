use crate::core::message::error::MessageError;

#[derive(Debug, Copy, Clone)]
pub enum TunnelError {
  MessageError(MessageError)
}

impl std::fmt::Display for TunnelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MessageError(e) => write!(f, "MessageError: {e}")
    }
  }
}

impl std::error::Error for TunnelError {}

impl From<MessageError> for TunnelError {
  fn from(error: MessageError) -> Self {
    Self::MessageError(error)
  }
}