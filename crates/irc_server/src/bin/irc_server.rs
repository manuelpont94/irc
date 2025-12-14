use flexi_logger::{Duplicate, Logger};
use irc_server::handlers::client::handle_client;
use irc_server::server_state::ServerState;
use log::info;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("debug")
        .and_then(|op| // log level||
        op.log_to_stderr() // output to stderr
        .duplicate_to_stderr(Duplicate::All) // duplicate all logs to stderr (optional)
        .start())
        .ok();
    let listener = TcpListener::bind("127.0.0.1:6667").await?;
    let server_state = ServerState::new();

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Client connected: {addr:?}");
        let state = server_state.clone();
        tokio::spawn(async move {
            handle_client(socket, addr, &state).await;
        });
    }
}
