use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::watch;
use crate::config::timeout::TUNNEL_CLIENT_HEARTBEAT_TIMEOUT;
use crate::core::message::message::{Message, MessageType};
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::model::{Flags, TunnelClient};

pub async fn tunnel_client_control(mut flags: Flags, tunnel_client: Arc<TunnelClient>) -> Result<(), TunnelError> {
  let mut buffer = [0u8; 1024];
  let (heartbeat_tx, heartbeat_rx) = watch::channel(false);

  let tunnel_client_heartbeat_thread = tokio::spawn(
    tunnel_client_heartbeat(
      flags.clone(),
      tunnel_client.clone(),
      (heartbeat_tx.clone(), heartbeat_rx)
    )
  );

  loop {
    let read_future = async {
      buffer = [0u8; 1024];
      let mut guard = tunnel_client.stream.lock().await;
      guard.read(buffer.as_mut()).await
    };

    select! {
      result = read_future => {
        match result {
          Ok(bytes_read) => {
            let message = Message::from_bytes(buffer.as_ref(), bytes_read);
            if let Ok(message) = message {
              match message.message_type {
                MessageType::Heartbeat => {
                  heartbeat_tx.send_replace(true);
                },
                MessageType::Service => {

                },
                MessageType::Proxy => {

                },
                MessageType::Authentication => {

                },
                MessageType::Port => {

                },
                MessageType::Close => {
                  flags.client_kill_tx.send_replace(true);
                  break;
                }
                MessageType::Error => {
                  flags.client_kill_tx.send_replace(true);
                  break;
                }
              }
            } else {
              flags.client_kill_tx.send_replace(true);
              break;
            }
          },
          Err(_error) => {
            //  TODO log
            flags.client_kill_tx.send_replace(true);
            break;
          },
        }
      },
      global_changed = flags.global_kill_rx.changed() => {
        if global_changed.is_err() || *flags.global_kill_rx.borrow() {
          flags.client_kill_tx.send_replace(true);
          break;
        } else {
          continue;
        }
      },
      client_changed = flags.client_kill_rx.changed() => {
        if client_changed.is_err() || *flags.client_kill_rx.borrow() {
          break;
        } else {
          continue;
        }
      },
    }
  }
  
  let _ = tunnel_client_heartbeat_thread.await;

  //  TODO log("Connection with client {client.addr} has ended")

  Ok(())
}

pub async fn tunnel_client_heartbeat(mut flags: Flags, tunnel_client: Arc<TunnelClient>, (heartbeat_tx, mut heartbeat_rx): (watch::Sender<bool>, watch::Receiver<bool>)) {
  let message = Message::new(MessageType::Heartbeat, String::new());

  loop {
    let value = select! {
      heartbeat_changed = heartbeat_rx.changed() => {
        if !heartbeat_changed.is_err() {
          Some(*heartbeat_rx.borrow())
        } else {
          flags.client_kill_tx.send_replace(true);
          None
        }
      },
      _global_changed = flags.global_kill_rx.changed() => None,
      _client_changed = flags.client_kill_rx.changed() => None,
      _sleep = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => None
    };

    match value {
      Some(value) if value => {
        select! {
          _global_changed = flags.global_kill_rx.changed() => { break; },
          _client_changed = flags.client_kill_rx.changed() => { break; },
          _sleep = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => {},
        }
      }
      _ => {
        break;
      }
    }

    let write_future = async {
      let mut guard = tunnel_client.stream.lock().await;
      heartbeat_tx.send_replace(false);
      heartbeat_rx.borrow_and_update();
      guard.write_all(message.to_vec().as_slice()).await
    };

    select! {
      write_result = write_future => {
        match write_result {
          Ok(_) => {},
          Err(_error) => {
            //  TODO log
            flags.client_kill_tx.send_replace(true);
            break;
          }
        }
      },
      _global_changed = flags.global_kill_rx.changed() => { break; },
      _client_changed = flags.client_kill_rx.changed() => { break; },
    }
  }
}