use crate::actors::Listener;
use anyhow::Result;

use lib_wc::sync::ShutdownController;

use tokio::select;

mod actors;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let shutdown = ShutdownController::new();
    let mut listener = Listener::new(&shutdown).await?;

    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await
        }
        _ = tokio::spawn(async move { listener.listen().await }) => {}
    }

    Ok(())
}
