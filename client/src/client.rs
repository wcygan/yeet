use crate::args;
use crate::keyboard_input::recv_from_stdin;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use lib_wc::sync::ShutdownListener;
use std::io;
use std::io::{stdin, BufRead, Write};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::select;

pub struct Client {
    /// The address of this client
    #[allow(dead_code)]
    local_address: SocketAddr,
    /// The address of the chat server
    remote_address: SocketAddr,
    /// The UDP socket that reads and writes messages
    socket: Socket,
    /// The name of the user of this chat client
    name: String,
    /// The shutdown listener used to enable graceful shutdown
    listener: ShutdownListener,
}

impl Client {
    pub async fn new(listener: ShutdownListener) -> Result<Self> {
        let remote_address: SocketAddr = args::Args::parse().address.parse()?;
        let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
        let local_address = udp_socket.local_addr()?;
        let socket = Socket::new(udp_socket);
        let name = input_sync("enter your name: ")?;

        if name.is_empty() {
            return Err(anyhow::anyhow!("name cannot be empty"));
        }

        Ok(Self {
            local_address,
            remote_address,
            socket,
            name,
            listener,
        })
    }

    pub async fn process(&mut self) -> Result<()> {
        self.join_server().await?;

        let mut chan = recv_from_stdin();

        while !self.listener.is_shutdown() {
            select! {
                _ = self.listener.recv() => {
                    self.leave_server().await?;
                }
                line = chan.recv() => {
                    if let Some(s) = line {
                    // TODO: handle this with retry?
                    //       `backon` library?
                        let _ = self.socket.write::<ToServer>(
                            &ToServer::Message { message: s },
                            self.remote_address
                        ).await;
                    }
                }
                msg = self.socket.read::<FromServer>() => {
                    match msg {
                        Ok((FromServer::Message { message }, _addr)) => {
                            println!("{}", message)
                        },
                        Ok((FromServer::Shutdown, _addr)) => {
                            println!("the server told us to shutdown!");
                            return Ok(())
                        },
                        Ok((FromServer::Heartbeat, _addr)) => {
                            println!("heartbeat received")
                        },
                        Ok((FromServer::Ack, _addr)) => {
                            println!("ack")
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    async fn join_server(&mut self) -> Result<()> {
        let Client {
            socket, listener, ..
        } = self;

        let join = ToServer::join(self.name.clone());
        socket.write::<ToServer>(&join, self.remote_address).await?;

        let wait_for_response = tokio::time::timeout(Duration::from_secs(1), async move {
            match socket.read::<FromServer>().await {
                Ok((FromServer::Ack, _src)) => {
                    println!("Connection established!");
                    Ok::<(), anyhow::Error>(())
                }
                _ => Err(anyhow::anyhow!("Unable to connect to the server. Goodbye!")),
            }
        });

        select! {
            _ = listener.recv() => {
                println!("Shutting down")
            }
            res = wait_for_response => {
                if let Err(e) = res {
                    println!("Timeout expired");
                    return Err(e.into())
                }
            }
        }

        Ok(())
    }

    async fn leave_server(&mut self) -> Result<()> {
        let _ = self
            .socket
            .write::<ToServer>(&ToServer::Leave, self.remote_address)
            .await;
        Ok(())
    }
}

fn input_sync(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}
