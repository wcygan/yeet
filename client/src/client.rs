use common::Socket;
use std::net::SocketAddr;

struct Client {
    /// The address of this client
    local_address: SocketAddr,
    /// The address of the chat server
    remote_address: SocketAddr,
    /// The UDP socket that reads and writes messages
    socket: Socket,
    /// The name of the user of this chat client
    name: String,
}
