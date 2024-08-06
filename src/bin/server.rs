use space_build::GameServer;
use space_build::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let (_tx, mut game_server) = GameServer::new()?;
    game_server.run().await?;
    Ok(())
}
