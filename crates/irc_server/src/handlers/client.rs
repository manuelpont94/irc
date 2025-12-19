use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

use super::request::handle_request;
use crate::channels_models::SubscriptionControl;
use crate::errors::InternalIrcError;
use crate::message_models::DirectIrcMessage;
use crate::types::{ChannelName, ClientId};
use crate::user_state::UserStatus;
use crate::{server_state::ServerState, user_state::UserState};

// Define the size of the personal outbound channel
const OUTBOUND_CHANNEL_SIZE: usize = 32;
const CONTROL_CHANNEL_SIZE: usize = 4;

/// Refactored entry point for a new client connection
pub async fn handle_client(socket: TcpStream, addr: SocketAddr, server_state: &ServerState) {
    info!("Client connected: {:?}", addr);

    let (tx_outbound, rx_outbound) = mpsc::channel::<DirectIrcMessage>(OUTBOUND_CHANNEL_SIZE);
    let (tx_control, rx_control) = mpsc::channel::<SubscriptionControl>(CONTROL_CHANNEL_SIZE);
    let (tx_status, rx_status) = mpsc::channel::<UserStatus>(CONTROL_CHANNEL_SIZE);

    let user_state = UserState::new(addr, tx_outbound, tx_control, tx_status);
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
        rx_status,
    ));
}

async fn client_reader_task(
    reader: tokio::io::ReadHalf<TcpStream>,
    client_id: ClientId,
    server_state: ServerState,
    user_state: UserState,
) -> Result<(), InternalIrcError> {
    // Wrap the reader for line-based (IRC) protocol handling
    let mut buffered_reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    loop {
        // Asynchronously read one line (ending in \r\n)
        let _bytes_read = match buffered_reader.read_line(&mut line).await {
            Ok(0) | Err(_) => {
                // TODO: Handle QUIT/cleanup in ServerState
                break;
            }
            Ok(n) => n,
        };

        // Process the request line
        let request = line.trim();
        info!(">> incoming [{}] # {}", client_id, request);

        // This call is now handled inside the Reader task:
        match handle_request(request, client_id, &server_state, &user_state).await {
            Ok(UserStatus::Leaving(reason)) => {
                info!("[{client_id}] Client Quit with message :{reason:?}");
                let _ = user_state.tx_status.send(UserStatus::Leaving(reason)).await;
                info!("[{}] Client disconnected.", client_id);
                debug!("{server_state:?}");
                break;
            }
            Ok(_) => debug!("{server_state:?}"),
            Err(e) => error!("Err occured while dealing with request {request} with error {e}"),
        }
        // The handler's response logic (writing to the socket) must change!
        // Instead of writing to the socket, it must use the outbound channel.

        line.clear(); // Clear the buffer for the next line
    }

    Ok(())
}

async fn client_writer_task(
    mut writer: tokio::io::WriteHalf<TcpStream>,
    client_id: ClientId,
    mut rx_outbound: mpsc::Receiver<DirectIrcMessage>,
    mut rx_control: mpsc::Receiver<SubscriptionControl>,
    mut rx_status: mpsc::Receiver<UserStatus>,
) -> Result<(), std::io::Error> {
    // Single aggregated channel for ALL outgoing messages (broadcast + direct)
    let (tx_aggregated, mut rx_aggregated) = mpsc::channel::<DirectIrcMessage>(100);

    // Track spawned tasks for cleanup
    let mut subscription_tasks: HashMap<ChannelName, tokio::task::JoinHandle<()>> = HashMap::new();

    loop {
        tokio::select! {
            Some(msg) = rx_outbound.recv() => {
                info!(">> out [{client_id}] direct # {}", &msg.raw_line);
                if let Err(e) = writer.write_all(msg.raw_line.as_bytes()).await {
                    error!("[{}] Failed to write: {:?}", client_id, e);
                    break;
                }
            }

            Some(msg) = rx_aggregated.recv() => {
                info!(">> out [{client_id}] broadcast # {}", &msg.raw_line);
                if let Err(e) = writer.write_all(msg.raw_line.as_bytes()).await {
                    error!("[{}] Failed to write: {:?}", client_id, e);
                    break;
                }
            }

            Some(control) = rx_control.recv() => {
                match control {
                    SubscriptionControl::Subscribe { channel_name, receiver } => {
                        info!("[{client_id}] Subscribed to: {channel_name}");

                        // Spawn a task that forwards broadcast messages to aggregated channel
                        let tx = tx_aggregated.clone();
                        let name = channel_name.clone();
                        let client_id_copy = client_id;

                        let handle = tokio::spawn(async move {
                            let mut rx = receiver;
                            loop {
                                match rx.recv().await {
                                    Ok(channel_msg) => {
                                        // Convert ChannelMessage to IrcMessage if needed
                                        let irc_msg = DirectIrcMessage {sender: None, raw_line: channel_msg.raw_line };
                                        if tx.send(irc_msg).await.is_err() {
                                            debug!("[{client_id_copy}] Aggregated channel closed for {name}");
                                            break;
                                        }
                                    }
                                    Err(broadcast::error::RecvError::Lagged(n)) => {
                                        error!("[{client_id_copy}] Lagged on {name} by {n}");
                                    }
                                    Err(broadcast::error::RecvError::Closed) => {
                                        info!("[{client_id_copy}] Channel {name} closed");
                                        break;
                                    }
                                }
                            }
                        });

                        subscription_tasks.insert(channel_name, handle);
                    }
                    SubscriptionControl::Unsubscribe(name) => {
                        info!("[{client_id}] Unsubscribed from: {name}");
                        if let Some(handle) = subscription_tasks.remove(&name) {
                            handle.abort();
                        }
                    }
                }
            }

            Some(status) = rx_status.recv() => {
                match status {
                    UserStatus::Leaving(_reason) => break,
                    _ => ()
                }
            }
        }
    }

    // Cleanup: abort all subscription tasks
    for (_name, handle) in subscription_tasks {
        handle.abort();
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Writer task terminated",
    ))
}
