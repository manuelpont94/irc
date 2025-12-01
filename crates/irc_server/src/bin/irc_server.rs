use flexi_logger::{Duplicate, Logger};
use irc_server::channel_ops::{IrcChannelOperation, IrcInvalidChannelOperation};
use irc_server::commands::IrcUnknownCommand;
use irc_server::pre_registration::IrcCapPreRegistration;
use irc_server::registration::IrcConnectionRegistration;
use irc_server::state::ServerState;
use irc_server::users::UserState;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn decode_utf8(buf: &[u8]) -> Result<&str, std::str::Utf8Error> {
    Ok(std::str::from_utf8(buf)?)
}

async fn handle_request<'a>(
    request: &'a str,
    server: &ServerState,
    user: &UserState,
) -> Result<String, &'a str> {
    log::info!("{:?}", request);
    IrcCapPreRegistration::handle_command(request, "*")
        .or_else(|_| IrcConnectionRegistration::handle_command(request, server, user))
        .or_else(|_| IrcChannelOperation::handle_command(request))
        .or_else(|_| IrcInvalidChannelOperation::handle_command(request))
        .or_else(|_| IrcUnknownCommand::handle_command(request))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("info")
        .and_then(|op| // log level||
        op.log_to_stderr() // output to stderr
        .duplicate_to_stderr(Duplicate::All) // duplicate all logs to stderr (optional)
        .start())
        .ok();
    let listener = TcpListener::bind("127.0.0.1:6667").await?;
    let server_state = ServerState::new();

    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("Client connected: {:?}", addr);
        let state = server_state.clone();
        tokio::spawn(async move {
            let mut buf = [0; 512];
            let user = UserState::new();
            loop {
                // info!("client state : {:?}");
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        error!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                if let Ok(requests) = decode_utf8(&buf[..n]) {
                    for request in requests.lines() {
                        info!(">> incoming # {}", request);
                        match handle_request(request.trim(), &state, &user).await {
                            Ok(reply) => {
                                if reply.is_empty() {
                                    continue;
                                }
                                info!(">> out # {}", &reply);
                                socket
                                    .write_all(&format!("{}\r\n", reply).as_bytes())
                                    .await
                                    .ok();
                                ();
                                _ = socket.flush().await;
                            }
                            Err(e) => {
                                error!("error with the request; err = {}", e);
                                let reply = format!("");
                                socket.write_all(&reply.as_bytes()).await.ok();
                            }
                        }
                    }
                }
            }
        });
    }
}
