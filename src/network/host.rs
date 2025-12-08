use super::{Message, MessageQueue, NetBuf, SessId, MAGIC_N};
use anyhow::{anyhow, Result};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::str::FromStr;

pub struct Host {
    state: HostConnection,
    send: MessageQueue,
    recv: MessageQueue,
    list: TcpListener,
    tcp: Option<TcpStream>,
    addr: SocketAddr,
    session_id: SessId,
    buf: NetBuf,
}

enum HostConnection {
    Begin,
    HandshakeRead,
    HandshakeRespond,
    Connected,
}

impl Host {
    pub fn new(address: &str) -> Result<Self> {
        println!("{}", address);
        let list = TcpListener::bind(address)?;
        let session_id: SessId = rand::random();
        list.set_nonblocking(true)?;
        Ok(Self {
            state: HostConnection::Begin,
            list,
            session_id,
            buf: [0; 128],
            tcp: None,
            addr: SocketAddr::from_str(address)?,
            recv: MessageQueue::new(),
            send: MessageQueue::new(),
        })
    }
}

impl super::Connection for Host {
    fn poll(&mut self) -> Result<()> {
        self.state = match self.state {
            HostConnection::Begin => match self.list.accept() {
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(HostConnection::Begin)
                    } else {
                        Err(anyhow!("{e}"))
                    }
                }
                Ok((recv, peer)) => {
                    recv.set_nonblocking(true).unwrap();
                    self.tcp = Some(recv);
                    self.addr = peer;
                    Ok(HostConnection::HandshakeRead)
                }
            },
            HostConnection::HandshakeRead => {
                match self.tcp.as_mut().unwrap().read(&mut self.buf[..8]) {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            Ok(HostConnection::HandshakeRead)
                        } else {
                            Err(anyhow!("{e}"))
                        }
                    }
                    Ok(recieved) => {
                        if recieved != 5 {
                            return Err(anyhow!("handshake length was improper"));
                        }
                        if self.buf[0..4] != MAGIC_N {
                            return Err(anyhow!("handshake failed"));
                        }
                        if self.buf[4..5] != [0x02] {
                            return Err(anyhow!("handshake failed"));
                        }
                        self.buf.fill(0);
                        Ok(HostConnection::HandshakeRespond)
                    }
                }
            }
            HostConnection::HandshakeRespond => {
                self.buf[..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
                self.buf[4..8].copy_from_slice(&self.session_id);
                println!("sending session_id: {:?}", &[..8]);
                let res = self.tcp.as_mut().unwrap().write_all(&self.buf[..8]);
                match res {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            Ok(HostConnection::HandshakeRespond)
                        } else {
                            Err(anyhow!("{e}"))
                        }
                    }
                    Ok(_) => {
                        println!("host: handshake complete");
                        self.buf.fill(0);
                        Ok(HostConnection::Connected)
                    }
                }
            }
            HostConnection::Connected => {
                if let Some(msgs) = super::recv_messages(
                    self.tcp.as_mut().unwrap(),
                    &mut self.buf,
                    &self.session_id,
                )? {
                    for msg in msgs {
                        self.recv.push_back(msg);
                    }
                }
                while let Some(mess) = self.send.pop_front() {
                    super::send_message(self.tcp.as_mut().unwrap(), mess, &self.session_id)
                        .unwrap();
                }
                Ok(HostConnection::Connected)
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
}
