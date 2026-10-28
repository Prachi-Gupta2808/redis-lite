mod command;
mod store;

use command::{Command, CommandError};
use store::Store;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6380").await?;
    println!("mini-redis listening on 127.0.0.1:6380");

    let store = Store::new();
    spawn_sweeper(store.clone());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client: {addr}");

        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store).await {
                eprintln!("Error with {addr}: {e}");
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    store: Store,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let reply = match Command::parse(&line) {
            Ok(cmd) => execute(cmd, &store),
            Err(CommandError::Empty) => continue,
            Err(msg) => format!("ERR {msg}"),
        };
        writer.write_all(format!("{reply}\r\n").as_bytes()).await?;
    }

    println!("Client disconnected");
    Ok(())
}

fn execute(cmd: Command, store: &Store) -> String {
    match cmd {
        Command::Ping => "PONG".to_string(),
        Command::Set { key, value, ttl_secs } => {
            store.set(key, value, ttl_secs);
            "OK".to_string()
        }
        Command::Get { key } => store.get(&key).unwrap_or_else(|| "(nil)".to_string()),
        Command::Del { key } => i32::from(store.del(&key)).to_string(),
        Command::Exists { key } => i32::from(store.exists(&key)).to_string(),
        Command::Ttl { key } => store.ttl(&key).to_string(),
        Command::Expire { key, seconds } => i32::from(store.expire(&key, seconds)).to_string(),
    }
}

fn spawn_sweeper(store: Store) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let removed = store.purge_expired();
            if removed > 0 {
                println!("Sweeper removed {removed} expired key(s)");
            }
        }
    });
}