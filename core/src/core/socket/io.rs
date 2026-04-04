use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use crate::core::message::error::MessageError;
use crate::core::message::message::Message;

#[derive(Debug)]
pub enum Error {
  MessageError(MessageError),
  IoError(std::io::Error),
}

pub async fn read_message(stream: &mut TlsStream<TcpStream>, buffer: &mut [u8]) -> Result<Message, Error> {
  buffer.fill(0);
  
  let read_result = stream.read(buffer.as_mut()).await;
  
  match read_result {
    Ok(bytes_read) => {
      let message = Message::from_bytes(buffer, bytes_read);
      match message {
        Ok(message) => Ok(message),
        Err(error) => Err(Error::MessageError(error))
      }
    },
    Err(error) => {
      Err(Error::IoError(error))
    },
  }
  
}

pub async fn send_message(stream: &mut TlsStream<TcpStream>, message: &Message) -> Result<usize, Error> {
  let message_bytes = message.to_vec();
  
  match message_bytes {
    Ok(message_bytes) => {
      match stream.write_all(message_bytes.as_slice()).await {
        Ok(_) => Ok(message_bytes.len()),
        Err(error) => Err(Error::IoError(error))
      }
    },
    Err(error) => {
      Err(Error::MessageError(error))
    },
  }
}