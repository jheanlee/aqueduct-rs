use crate::core::error::MessageError;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Error {
  MessageError(MessageError)
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::MessageError(e) => write!(f, "MessageError: {e}")
    }
  }
}

impl std::error::Error for Error {}