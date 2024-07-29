use std::env;

use server::Error;
use server::Result;
use server::SpaceBuildGame;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let level = env::args().nth(1).map_or(Level::INFO, |value| {
        value.parse::<Level>().map_or(Level::INFO, |value| value)
    });

    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|_| Error::TracingError)?;

    let mut game = SpaceBuildGame::new()?;

    game.run_loop().await?;

    Ok(())
}
