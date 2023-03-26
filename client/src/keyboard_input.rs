use std::io::{stdin, BufRead, BufReader};

use tokio::sync::mpsc;

/// Create a channel that streams input from stdin.
///
/// Internally this creates a separate thread to block on stdin.
pub fn recv_from_stdin() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>(10);
    std::thread::spawn(move || block_on_stdin(tx));
    rx
}

fn block_on_stdin(tx: mpsc::Sender<String>) {
    loop {
        println!("recv");
        let reader = BufReader::new(stdin());
        let mut lines = reader.lines();
        if let Some(Ok(s)) = lines.next() {
            let _ = tx.blocking_send(s);
        }
    }
}
