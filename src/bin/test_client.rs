use space_build::player_client::PlayerClient;

extern crate tokio;
extern crate tokio_tungstenite;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut player = PlayerClient::connect("ws://127.0.0.1:2567".to_string()).await?;

    player.login("test_player".to_string()).await?;

    loop {
        let msg = player.wait_message().await?;
        tracing::info!("{}", msg);
    }
}
