use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

use super::request::handle_request;
use crate::errors::InternalIrcError;
use crate::message_models::{IrcMessage, SubscriptionControl};
use crate::{server_state::ServerState, user_state::UserState};

// Define the size of the personal outbound channel
const OUTBOUND_CHANNEL_SIZE: usize = 32;
const CONTROL_CHANNEL_SIZE: usize = 4;

/// Refactored entry point for a new client connection
pub async fn handle_client(socket: TcpStream, addr: SocketAddr, server_state: &ServerState) {
    info!("Client connected: {:?}", addr);

    let (tx_outbound, rx_outbound) = mpsc::channel::<IrcMessage>(OUTBOUND_CHANNEL_SIZE);
    let (tx_control, rx_control) = mpsc::channel::<SubscriptionControl>(CONTROL_CHANNEL_SIZE);

    let user_state = UserState::new(addr, tx_outbound, tx_control);
    let client_id = match server_state.add_connecting_user(&user_state).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to register user: {e:?}");
            return;
        }
    };

    let (read_half, write_half) = io::split(socket);

    // 4. Spawn two new, independent tasks
    tokio::spawn(client_reader_task(
        read_half,
        client_id,
        server_state.clone(),
        user_state.clone(),
    ));
    tokio::spawn(client_writer_task(
        write_half,
        client_id,
        rx_outbound,
        rx_control,
    ));
}

fn decode_utf8(buf: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(buf)
}

async fn client_reader_task(
    reader: tokio::io::ReadHalf<TcpStream>,
    client_id: usize,
    server_state: ServerState,
    user_state: UserState,
) -> Result<(), InternalIrcError> {
    // Wrap the reader for line-based (IRC) protocol handling
    let mut buffered_reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    loop {
        // Asynchronously read one line (ending in \r\n)
        let bytes_read = match buffered_reader.read_line(&mut line).await {
            Ok(0) | Err(_) => {
                info!("[{}] Client disconnected.", client_id);
                // TODO: Handle QUIT/cleanup in ServerState
                break;
            }
            Ok(n) => n,
        };

        // Process the request line
        let request = line.trim();
        info!(">> incoming [{}] # {}", client_id, request);

        // This call is now handled inside the Reader task:
        let response = handle_request(request, &server_state, &user_state).await;

        // The handler's response logic (writing to the socket) must change!
        // Instead of writing to the socket, it must use the outbound channel.

        line.clear(); // Clear the buffer for the next line
    }

    Ok(())
}

async fn client_writer_task(
    mut writer: tokio::io::WriteHalf<TcpStream>,
    client_id: usize,
    mut rx_outbound: mpsc::Receiver<IrcMessage>, // Targeted replies
    mut rx_control: mpsc::Receiver<SubscriptionControl>, // Channel management
) -> Result<(), std::io::Error> {
    // Map to hold dynamic channel broadcast receivers
    let mut channel_subscriptions: HashMap<String, broadcast::Receiver<IrcMessage>> =
        HashMap::new();
    let mut write_err = false;

    loop {
        tokio::select! {
            Some(msg) = rx_outbound.recv() => {
                if let Err(e) = writer.write_all(msg.raw_line.as_bytes()).await {
                    error!("[{}] Failed to write targeted message: {:?}", client_id, e);
                    write_err = true;
                }
            }

            Some(control) = rx_control.recv() => {
                match control {
                    SubscriptionControl::Subscribe { channel_name, receiver } => {
                        info!("[{client_id}] Subscribed to: {channel_name}");
                        channel_subscriptions.insert(channel_name, receiver);
                    }
                    SubscriptionControl::Unsubscribe(name) => {
                        info!("[{client_id}] Unsubscribed from: {name}");
                        channel_subscriptions.remove(&name);
                    }
                }
            }

            else => {
                if write_err { break; } // Break on error

                tokio::task::yield_now().await;
            }
        }

        // --- Processing Dynamic Broadcasts (Draining the Receivers) ---
        let mut messages_to_write = Vec::new();
        let mut channels_to_remove = Vec::new();

        for (channel_name, receiver) in channel_subscriptions.iter_mut() {
            match receiver.try_recv() {
                Ok(msg) => {
                    messages_to_write.push(msg);
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    error!("[{client_id}] Lagged on channel {channel_name}");
                    // Critical: Client is too slow and missed a message.
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    // Channel was destroyed (e.g., last user PARTed, server cleanup)
                    channels_to_remove.push(channel_name.clone());
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    // Nothing to read, continue.
                }
            }
        }

        // Clean up closed channels
        for name in channels_to_remove {
            channel_subscriptions.remove(&name);
        }

        // Write all collected broadcast messages to the socket
        if !messages_to_write.is_empty() {
            for msg in messages_to_write {
                if let Err(e) = writer.write_all(msg.raw_line.as_bytes()).await {
                    error!("[{client_id}] Failed to write broadcast message: {e:?}");
                    write_err = true;
                    break;
                }
            }
            if write_err {
                break;
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Writer task terminated",
    ))
}
