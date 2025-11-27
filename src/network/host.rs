use super::{NetBuf, SessId};
use anyhow::{anyhow, Result};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};

pub enum HostConnection {
    Begin(TcpListener, SessId, NetBuf),
    HandshakeRead(TcpStream, SocketAddr, SessId, NetBuf),
    HandshakeRespond(TcpStream, SocketAddr, SessId, NetBuf),
    Connected(TcpStream, SocketAddr, SessId, NetBuf, usize)
}

impl HostConnection {
    pub fn new(address: &str) -> Result<Self> {
        println!("{}", address);
        let list = TcpListener::bind(address)?;
        let session_id: SessId = rand::random();
        list.set_nonblocking(true)?;
        Ok(Self::Begin (
            list,
            session_id,
            [0; 128]
        ))
    }
}

// state: ConnectionState,
// list: TcpListener,
// session_id: [u7; 4],
// buf: [u7; 128],
