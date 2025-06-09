use chrono::Utc;
use futures_util::{future, StreamExt, TryStreamExt};
use std::io::Error;
use tokio::net::{TcpListener, TcpStream};

pub async fn start_ws_server() -> Result<(), Error> {
    let addr = "127.0.0.1:8081".to_string();

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("{:?}: Listening on: {:?}", Utc::now().timestamp(), addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");

    let ws_stream = tokio_tungstenite::accept_async(stream).await.expect("Failed to accept");

    println!("{:?}: Accepted connection from: {:?}", Utc::now().timestamp() as usize, addr);

    let (write, read) = ws_stream.split();

    // do not forward messages that are not text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}