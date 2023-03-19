pub use connection::UdpConnection;
pub use event::{FromServer, ToServer};
mod connection;
mod event;

static UDP_PACKET_SIZE: u16 = 512;
