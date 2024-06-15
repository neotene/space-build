use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::stream::StreamExt;
use futures::sink::SinkExt;

#[tokio::main]
async fn main() {
    // Bind the TCP listener to the specified address
    let addr = "127.0.0.1:2567";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        // Spawn a new task to handle the WebSocket connection
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake");

            println!("New WebSocket connection");

            let (mut write, mut read) = ws_stream.split();

            // Send "hello" message to the client
            write.send(Message::Text("hello".to_string()))
                .await
                .expect("Failed to send message");

            // Optional: Handle incoming messages from the client
            while let Some(Ok(msg)) = read.next().await {
                println!("Received a message from client: {:?}", msg);
            }
        });
    }
}
