use log::{debug, error, info};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::request::handle_request;
use crate::{server_state::ServerState, users::UserState};

pub async fn handle_client(mut socket: TcpStream, addr: SocketAddr, server_state: &ServerState) {
    info!("Client connected: {:?}", addr);

    let user_state = UserState::new(addr);
    let mut buf = [0; 512];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => return, // socket closed
            Ok(n) => n,
            Err(e) => {
                error!("failed to read from socket; err = {:?}", e);
                return;
            }
        };
        debug!("{user_state:?}");
        debug!("{server_state:?}");

        if let Ok(requests) = decode_utf8(&buf[..n]) {
            for request in requests.lines() {
                info!(">> incoming # {}", request);

                match handle_request(request.trim(), server_state, &user_state).await {
                    Ok(Some(reply)) => {
                        info!(">> out # {}", &reply);
                        if let Err(e) = socket.write_all(format!("{reply}\r\n").as_bytes()).await {
                            error!("failed to write to socket: {:?}", e);
                            return;
                        }
                        let _ = socket.flush().await;
                    }
                    Ok(None) => (), // No response expected
                    Err(e) => {
                        error!("error with the request; err = {}", e);
                    }
                }
            }
        }
    }
}

fn decode_utf8(buf: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(buf)
}
