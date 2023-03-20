use crate::ToServer::{Join, Message};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ToServer {
    Join { name: String },
    Message { message: String },
    Leave,
    Pong,
}

impl ToServer {
    pub fn join(name: String) -> ToServer {
        Join { name }
    }

    pub fn message(message: String) -> ToServer {
        Message { message }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FromServer {
    Message { message: String },
    Ack,
    Ping,
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UDP_PACKET_SIZE;

    #[test]
    fn size_of_to_server() {
        // The size needs to be smaller than the size of a UDP packet
        let size = std::mem::size_of::<ToServer>();
        assert!(size <= UDP_PACKET_SIZE as usize)
    }

    #[test]
    fn size_of_from_server() {
        // The size needs to be smaller than the size of a UDP packet
        let size = std::mem::size_of::<FromServer>();
        assert!(size <= UDP_PACKET_SIZE as usize)
    }
}
