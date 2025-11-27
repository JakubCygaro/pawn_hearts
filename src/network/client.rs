use super::{NetBuf, NetworkConnection, SessId};
use anyhow::{anyhow, Result};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::str::FromStr;

pub enum ClientConnection {
    Begin(TcpStream, SocketAddr, NetBuf),
    // HandshakeWrite(TcpStream, SocketAddr, SessId, NetBuf),
    SessionIdReak(TcpStream, SocketAddr, NetBuf),
    // HandshakeRespond(TcpStream, SocketAddr, SessId, NetBuf),
    Connected(TcpStream, SocketAddr, SessId, NetBuf, usize),
}

impl ClientConnection {
    pub fn new(address: &str) -> Result<Self> {
        println!("{}", address);
        let list = TcpStream::connect(address)?;
        list.set_nonblocking(true)?;
        Ok(Self::Begin(list, SocketAddr::from_str(address)?, [0; 128]))
    }
}
