use crate::client::Client;

use anyhow::Result;

use shutdown_async::ShutdownController;

use tokio::select;

mod args;
mod client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let shutdown = ShutdownController::new();
    let mut client = Client::new(shutdown.subscribe()).await?;

    select! {
        _ = tokio::signal::ctrl_c() => {
            println!("shutting down");
            shutdown.shutdown().await;
            println!("Done!");
        },
        res = tokio::spawn(async move { client.process().await }) => {
            return res?
        }
    }

    Ok(())
}
