use crate::UDP_PACKET_SIZE;
use anyhow::Result;
use bincode::{deserialize, serialize};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// A high-level UDP Socket that allows for writing and reading
/// serializable types
pub struct Socket {
    buffer: [u8; 512],
    socket: UdpSocket,
}

impl Socket {
    pub fn new(socket: UdpSocket) -> Self {
        let buffer = [0; 512];
        Self { buffer, socket }
    }

    pub async fn write<T: Serialize>(&mut self, value: &T, addr: SocketAddr) -> Result<()> {
        let buf = serialize(value)?;
        self.socket.send_to(buf.as_slice(), addr).await?;
        Ok(())
    }

    pub async fn read<T: DeserializeOwned>(&mut self) -> Result<(T, SocketAddr)> {
        let (size, src) = self.socket.recv_from(&mut self.buffer).await?;
        assert!(size <= UDP_PACKET_SIZE as usize);
        let t = deserialize::<T>(self.buffer.as_slice())?;
        Ok((t, src))
    }
}
