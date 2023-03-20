use crate::keyboard_input::recv_from_stdin;
use std::io::{stdin, BufRead, BufReader};
use tokio::select;
use tokio::sync::mpsc;

mod keyboard_input;

#[tokio::main]
async fn main() {
    let mut chan = recv_from_stdin();

    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Done");
                return;
            }
            line = chan.recv() => {
                match line {
                    Some(s) => println!("{s}"),
                    None => {}
                }
            }
        }
    }
}
