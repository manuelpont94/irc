use std::sync::Arc;

use clap::Parser;
use flexi_logger::{Duplicate, Logger};
use irc_server::config::Config;
use irc_server::constants::SERVER_NAME;
use irc_server::handlers::client::handle_client;
use irc_server::server_state::ServerState;
use log::info;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(author, version, about = "A modest Rust IRC server")]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = Config::load(&args.config).expect("Failed to load config");
    SERVER_NAME
        .set(config.server.name)
        .expect("Server name already set!");
    Logger::try_with_str("debug")
        .and_then(|op| // log level||
        op.log_to_stderr() // output to stderr
        .duplicate_to_stderr(Duplicate::All) // duplicate all logs to stderr (optional)
        .start())
        .ok();
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.network.bind_address, config.network.port
    ))
    .await?;
    let server_state = Arc::new(ServerState::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Client connected: {addr:?}");
        let ip = addr.ip();
        let state = server_state.clone();
        // 1. Pre-check: Increment and validate
        {
            let mut count = server_state.ip_counts.entry(ip).or_insert(0);
            if *count >= config.limits.max_connections_per_ip {
                eprintln!("Rejecting IP {}: too many connections", ip);
                continue; // Drop the stream immediately
            }
            *count += 1;
        }
        tokio::spawn(async move {
            handle_client(socket, addr, &state).await;
        });
    }
}
