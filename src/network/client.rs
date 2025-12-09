use super::{Message, MessageQueue, NetBuf, SessId};
use anyhow::{anyhow, Result};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct Client {
    state: ClientConnection,
    send: MessageQueue,
    recv: MessageQueue,
    tcp: TcpStream,
    // addr: SocketAddr,
    session_id: SessId,
    buf: NetBuf,
}

pub enum ClientConnection {
    Begin,
    SessionIdRead,
    Connected,
}

impl Client {
    pub fn new(address: &str) -> Result<Self> {
        println!("{}", address);
        let tcp = TcpStream::connect(address)?;
        tcp.set_nonblocking(true)?;
        Ok(Self {
            state: ClientConnection::Begin,
            send: MessageQueue::new(),
            recv: MessageQueue::new(),
            tcp,
            // addr: SocketAddr::from_str(address)?,
            session_id: [0; 4],
            buf: [0; 128],
        })
    }
}

impl super::Connection for Client {
    // fn state(&self);
    fn poll(&mut self) -> Result<()> {
        self.state = match self.state {
            ClientConnection::Begin => match self.tcp.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0x02]) {
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(ClientConnection::Begin)
                    } else {
                        Err(anyhow!("{e}"))
                    }
                }
                Ok(_) => Ok(ClientConnection::SessionIdRead),
            },
            ClientConnection::SessionIdRead => match self.tcp.read(&mut self.buf[..8]) {
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(ClientConnection::SessionIdRead)
                    } else {
                        Err(anyhow!("{e}"))
                    }
                }
                Ok(recieved) => {
                    if recieved != 8 {
                        return Err(anyhow!("handshake length failure"));
                    }
                    if self.buf[..4] != [0xDE, 0xAD, 0xBE, 0xEF] {
                        return Err(anyhow!("handshake header failure"));
                    }
                    let id: SessId = self.buf[4..8].try_into().unwrap();
                    self.session_id = id;
                    println!("session_id: {:?}", id);
                    self.buf.fill(0);
                    Ok(ClientConnection::Connected)
                }
            },
            ClientConnection::Connected => {
                if let Some(msgs) =
                    super::recv_messages(&mut self.tcp, &mut self.buf, &self.session_id)?
                {
                    for msg in msgs {
                        self.recv.push_back(msg);
                    }
                }
                while let Some(mess) = self.send.pop_front() {
                    super::send_message(&mut self.tcp, mess, &self.session_id).unwrap();
                }
                Ok(ClientConnection::Connected)
            }
        }?;
        Ok(())
    }
    fn send(&mut self, msg: Message) {
        self.send.push_back(msg);
    }
    fn recv(&mut self) -> Option<Message> {
        self.recv.pop_front()
    }
    fn is_connected(&self) -> bool {
        matches!(&self.state, ClientConnection::Connected)
    }
}
