use crate::client::Client;

use anyhow::Result;

use lib_wc::sync::ShutdownController;

use tokio::select;

mod args;
mod client;
mod keyboard_input;

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
