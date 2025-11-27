use crate::board::{BoardMove, BoardPos};
use anyhow::{anyhow, bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::io::ErrorKind;
use std::net::{TcpListener, UdpSocket};

pub type SessId = [u8; 4];
pub type NetBuf = [u8; 128];
pub const MAGIC_N: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
pub mod host;



#[derive(PartialEq, Eq)]
pub enum ConnectionVariant {
    Host,
    Client,
}
#[derive(PartialEq, Eq)]
enum ConnectionState {
    NotConnected,
    WaitingOnClient,
    WaitingOnHost,
    Connected,
}
pub struct NetworkConnection {
    state: ConnectionState,
    variant: ConnectionVariant,
    sock: std::net::UdpSocket,
    session_id: [u8; 4],
    buf: [u8; 128],
}
impl NetworkConnection {
    pub fn host(address: &str) -> Result<Self> {
        println!("{}", address);
        let conn = UdpSocket::bind(address)?;
        let session_id: [u8; 4] = rand::random();
        conn.set_nonblocking(true)?;
        Ok(Self {
            state: ConnectionState::NotConnected,
            variant: ConnectionVariant::Host,
            sock: conn,
            session_id,
            buf: [0u8; 128],
        })
    }
    pub fn accept_connection(&mut self) -> Result<Option<()>> {
        if self.variant != ConnectionVariant::Host {
            anyhow::bail!("A client is not a host")
        };
        match self.state {
            ConnectionState::NotConnected => {
                let (recieved, peer) = match self.sock.recv_from(&mut self.buf[..8]) {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            return Ok(None);
                        } else {
                            bail!(e)
                        }
                    }
                    Ok((recv, peer)) => (recv, peer),
                };
                println!("recieved: {} buf: {:?}", recieved, self.buf);
                if recieved != 5 {
                    anyhow::bail!("handshake length was improper");
                }
                if self.buf[0..5] != [0xDE, 0xAD, 0xBE, 0xEF, 0x02] {
                    anyhow::bail!("handshake failed");
                }
                self.sock.connect(peer)?;
                self.state = ConnectionState::WaitingOnClient;
                Ok(None)
            }
            ConnectionState::WaitingOnClient => {
                self.buf[..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
                self.buf[4..8].copy_from_slice(&self.session_id);
                println!("sending session_id: {:?}", &self.buf[..8]);
                let res = self.sock.send(&self.buf[..8]);
                if let Err(e) = res {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.state = ConnectionState::Connected;
                        self.buf.iter_mut().for_each(|b| *b = 0);
                        return Ok(Some(()));
                    } else {
                        anyhow::bail!(e)
                    }
                }
                self.state = ConnectionState::Connected;
                self.buf.iter_mut().for_each(|b| *b = 0);
                Ok(Some(()))
            }
            _ => {
                anyhow::bail!("connection in invalid state")
            }
        }
    }
    pub fn client(host_adress: &str) -> Result<Self> {
        let conn = UdpSocket::bind("127.0.0.1:0")?;
        println!("client bound to : {}", conn.local_addr()?);
        conn.connect(host_adress)?;
        println!("client connected to : {}", conn.peer_addr()?);

        conn.set_nonblocking(true)?;
        Ok(Self {
            state: ConnectionState::NotConnected,
            variant: ConnectionVariant::Client,
            sock: conn,
            session_id: [0u8; 4],
            buf: [0u8; 128],
        })
    }
    pub fn client_connect(&mut self) -> Result<Option<()>> {
        if self.variant != ConnectionVariant::Client {
            anyhow::bail!("Host is not a client")
        };
        match self.state {
            ConnectionState::NotConnected => {
                match self.sock.send(&[0xDE, 0xAD, 0xBE, 0xEF, 0x02]) {
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            bail!(e)
                        } else {
                            Ok(None)
                        }
                    }
                    Ok(_) => {
                        self.state = ConnectionState::WaitingOnHost;
                        Ok(None)
                    }
                }
            }
            ConnectionState::WaitingOnHost => {
                let recieved = match self.sock.recv(&mut self.buf[..8]) {
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            bail!(e)
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(r) => r,
                };
                if recieved != 8 {
                    bail!("handshake length failure")
                }
                if self.buf[..4] != [0xDE, 0xAD, 0xBE, 0xEF] {
                    bail!("handshake header failure")
                }
                self.session_id = self.buf[4..8].try_into()?;
                self.buf.iter_mut().for_each(|b| *b = 0);
                self.state = ConnectionState::Connected;
                println!("session_id: {:?}", self.session_id);
                Ok(Some(()))
            }
            _ => {
                bail!("connection in invalid state")
            }
        }
    }
    pub fn recv(&mut self) -> Result<Option<Message>> {
        if self.state != ConnectionState::Connected {
            bail!("socket not connected with a peer")
        }
        match self.sock.recv(&mut self.buf) {
            Ok(n) => {
                println!("recieved something");
                if n < 5 {
                    return Ok(None);
                };
                if self.buf[..4] != self.session_id {
                    return Ok(None);
                }
                if let Ok(msg) = decode_message(&self.buf[4..n]) {
                    self.buf.fill(0);
                    Ok(Some(msg))
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(other) => Err(other.into()),
        }
    }
    pub fn send(&mut self, msg: Message) -> Result<Option<()>> {
        if self.state != ConnectionState::Connected {
            bail!("socket not connected with a peer")
        }
        let mut bytes = BytesMut::new();
        bytes.put(self.session_id.as_slice());
        let encoded_message = encode_message(&msg);
        let encoded_message = encoded_message.iter().as_slice();
        bytes.put(encoded_message);
        match self.sock.send(&bytes) {
            Ok(_) => Ok(Some(())),
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => bail!(e),
        }
    }
}
fn decode_message(bytes: &[u8]) -> Result<Message> {
    match bytes[0] {
        0x01 if bytes.len() == 7 => {
            let mess = Message::Moved(BoardMove::new(
                BoardPos {
                    row: bytes[1] as usize,
                    col: bytes[2] as usize,
                },
                BoardPos {
                    row: bytes[3] as usize,
                    col: bytes[4] as usize,
                },
            ));
            Ok(mess)
        }
        _ => Err(anyhow!("invalid message kind")),
    }
}
#[derive(Debug)]
pub enum Message {
    Moved(super::board::BoardMove), // 0x01
}
fn encode_message(msg: &Message) -> Bytes {
    let mut bytes = BytesMut::new();
    match *msg {
        Message::Moved(m) => {
            bytes.put_u8(0x01);
            bytes.put(m.to_bytes());
        }
    }
    bytes.into()
}
