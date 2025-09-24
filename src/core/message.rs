use crate::core::error::MessageError;
use crate::core::error::MessageError::{InvalidString, InvalidType, MessageEmpty, MessageTooLong};

static MAX_MESSAGE_LEN: usize = 256;

pub(crate) enum MessageType {
  Heartbeat,
  Service,    //  service connection
  Proxy,      //  proxy connection
  Authentication,

  Port,

  Close,
  Error,
}
impl MessageType {
  pub(crate) fn as_u8(&self) -> u8 {
    match self {
      Self::Heartbeat => 0x10,
      Self::Service => 0x11,
      Self::Proxy => 0x12,
      Self::Authentication => 0x13,
      Self::Port => 0x20,
      Self::Close => 0xf0,
      Self::Error => 0xff,
    }
  }

  pub(crate) fn from_u8(message_type: u8) -> Result<Self, MessageError> {
    match message_type {
      0x10 => Ok(Self::Heartbeat),
      0x11 => Ok(Self::Service),
      0x12 => Ok(Self::Proxy),
      0x13 => Ok(Self::Authentication),
      0x20 => Ok(Self::Port),
      0xf0 => Ok(Self::Close),
      0xff => Ok(Self::Error),
      _ => Err(InvalidType)
    }
  }
}

pub(crate) struct Message {
  pub message_type: MessageType,
  pub message_string: String,
}

impl Message {
  pub(crate) fn new(message_type: MessageType, message_string: String) -> Self {
    Self {message_type, message_string}
  }

  pub(crate) fn to_vec(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.push(self.message_type.as_u8());
    buffer.extend(self.message_string.as_bytes());
    buffer
  }

  pub(crate) fn from_vec(vec: &Vec<u8>) -> Result<Self, MessageError> {
    if !vec.is_empty() {
      if vec.len() <= MAX_MESSAGE_LEN - 1 {
        Ok(
          Self {
            message_type: MessageType::from_u8(vec[0])?,
            message_string: if vec.len() > 1 {
              std::str::from_utf8(&vec[1..]).map_err(|_| InvalidString)?.to_string()
            } else { Default::default() }
          }
        )
      } else {
        Err(MessageTooLong)
      }
    } else {
      Err(MessageEmpty)
    }
  }
}