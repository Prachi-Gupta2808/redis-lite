use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6380").await?;
    println!("mini-redis listening on 127.0.0.1:6380");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client: {addr}");

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error with {addr}: {e}");
            }
        });
    }
}

async fn handle_client(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        writer.write_all(format!("{line}\n").as_bytes()).await?;
    }

    println!("Client disconnected");
    Ok(())
}