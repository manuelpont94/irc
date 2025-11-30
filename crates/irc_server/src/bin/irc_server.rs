use dashmap::{DashMap, DashSet};
use flexi_logger::{Duplicate, Logger};
use irc_server::channels::{ChannelName, IrcChannel};
use irc_server::commands::{
    IrcChannelOperation, IrcConnectionRegistration, IrcInvalidChannelOperation, IrcUnknownCommand,
};
use irc_server::users::{User, UserId};
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct UserState(Arc<Mutex<Client>>);
impl UserState {
    pub fn new() -> Self {
        UserState(Arc::new(Mutex::new(Client::default())))
    }

    pub async fn with_nick(&self, nick: String) {
        let mut user_data = self.0.lock().await;
        user_data.nick = Some(nick);
        _ = self.is_registered();
    }

    pub async fn with_user(&self, user: String) {
        let mut user_data = self.0.lock().await;
        user_data.user = Some(user);
        _ = self.is_registered();
    }

    pub async fn is_registered(&self) -> bool {
        let mut user_data = self.0.lock().await;
        if user_data.registered {
            true
        } else if user_data.nick.is_none() || user_data.user.is_none() {
            false
        } else {
            user_data.registered = true;
            true
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: Arc<DashMap<String, IrcChannel>>,
    pub users: Arc<DashMap<UserId, User>>,
    // pub registering_users: Arc<DashSet<>>
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: Arc::new(DashMap::<ChannelName, IrcChannel>::new()),
            users: Arc::new(DashMap::<UserId, User>::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    nick: Option<String>,
    user: Option<String>,
    registered: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            nick: None,
            user: None,
            registered: false,
        }
    }
}

fn decode_utf8(buf: &[u8]) -> Result<&str, std::str::Utf8Error> {
    Ok(std::str::from_utf8(buf)?)
}

async fn handle_request<'a>(
    request: &'a str,
    state: &ServerState,
    client: &UserState,
) -> Result<String, &'a str> {
    log::info!("{:?}", request);
    IrcConnectionRegistration::handle_command(request)
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
                        info!(">> incoming {}", request);
                        match handle_request(request.trim(), &state, &user).await {
                            Ok(reply) => {
                                info!(">>{}", &reply);
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
