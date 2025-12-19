use std::error::Error;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    time::sleep(Duration::from_millis(3000)).await;
    let server_addr = "127.0.0.1:6667";
    let num_clients = 1000; // Total virtual users
    let interval_ms = 1000; // 1 second interval

    println!("Starting stress test: {} clients...", num_clients);

    for i in 0..num_clients {
        tokio::spawn(async move {
            if let Err(e) = run_client(i, server_addr, interval_ms).await {
                eprintln!("Client {} error: {}", i, e);
            }
        });

        // Small delay between spawns to avoid overwhelming the OS accept() queue
        time::sleep(Duration::from_millis(5)).await;
    }

    // Keep the main thread alive forever
    std::future::pending::<()>().await;
    Ok(())
}

async fn run_client(id: usize, addr: &str, interval: u64) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    let nick = format!("bot{}", id);

    // 1. Handshake
    let login = format!("NICK {}\r\nUSER {} 0 * :LoadTester\r\n", nick, nick);
    stream.write_all(login.as_bytes()).await?;

    // 2. Join a common channel to test the Broadcast Multiplier
    stream.write_all(b"JOIN #stress_test\r\n").await?;

    let mut ticker = time::interval(Duration::from_millis(interval));
    let mut cpt = 0_usize;
    loop {
        ticker.tick().await;
        let msg = format!(
            "PRIVMSG #stress_test :Message from {} - Load Testing...{cpt}\r\n",
            nick
        );
        cpt += 1;
        if let Err(_) = stream.write_all(msg.as_bytes()).await {
            break; // Connection lost
        }
    }
    Ok(())
}
