use crate::board::{BoardMove, BoardPos};
use anyhow::{anyhow, bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};

pub type SessId = [u8; 4];
pub type NetBuf = [u8; NETBUF_SIZE];
pub const MAGIC_N: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
pub const NETBUF_SIZE: usize = 128;
pub type MessageQueue = VecDeque<Message>;
pub mod client;
pub mod host;

pub fn recv_message(
    sock: &mut TcpStream,
    buf: &mut NetBuf,
    session_id: &SessId,
) -> Result<Option<Message>> {
    match sock.read(buf) {
        Ok(n) => {
            println!("recieved something");
            if n < 5 {
                return Ok(None);
            };
            if buf[..4] != *session_id {
                return Ok(None);
            }
            if let Ok(msg) = decode_message(&buf[4..n]) {
                buf.fill(0);
                Ok(Some(msg))
            } else {
                Ok(None)
            }
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
        Err(other) => Err(other.into()),
    }
}
pub fn send(sock: &mut TcpStream, msg: Message, session_id: &SessId) -> Result<Option<()>> {
    let mut bytes = BytesMut::new();
    bytes.put(session_id.as_slice());
    let encoded_message = encode_message(&msg);
    let encoded_message = encoded_message.iter().as_slice();
    bytes.put(encoded_message);
    match sock.write(&bytes) {
        Ok(_) => Ok(Some(())),
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e.into()),
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
        0x02 if bytes.len() == 1 => {
            Ok(Message::Rejected())
        }
        0x03 if bytes.len() == 1 => {
            Ok(Message::Accepted())
        }
        _ => Err(anyhow!("invalid message kind")),
    }
}
#[derive(Debug)]
pub enum Message {
    Moved(super::board::BoardMove), // 0x01
    Rejected(), // 0x02
    Accepted(), // 0x03
}
fn encode_message(msg: &Message) -> Bytes {
    let mut bytes = BytesMut::new();
    match *msg {
        Message::Moved(m) => {
            bytes.put_u8(0x01);
            bytes.put(m.to_bytes());
        }
        Message::Rejected() => {
            bytes.put_u8(0x02);
        }
        Message::Accepted() => {
            bytes.put_u8(0x03);
        }
    }
    bytes.into()
}
