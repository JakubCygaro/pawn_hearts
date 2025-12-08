use crate::board::{BoardMove, BoardPos};
use anyhow::{anyhow, bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};

pub mod client;
pub mod host;

pub type SessId = [u8; 4];
pub type NetBuf = [u8; NETBUF_SIZE];
pub const MAGIC_N: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
pub const NETBUF_SIZE: usize = 128;
pub type MessageQueue = VecDeque<Message>;

pub trait Connection {
    // fn state(&self);
    fn poll(&mut self) -> Result<()>;
    fn send(&mut self, msg: Message);
    fn recv(&mut self) -> Option<Message>;
}

pub fn recv_messages(
    sock: &mut TcpStream,
    buf: &mut NetBuf,
    session_id: &SessId,
) -> Result<Option<Vec<Message>>> {
    let mut ret = vec![];
    match sock.read(buf) {
        Ok(n) => {
            println!("recieved something: {:?}\nn: {n}", buf);
            let mut cursor = 0;
            while cursor < n {
                // would be too short for a valid message
                if n - cursor < 5 {
                    return Ok(None);
                };
                // ensure the session id matches
                if buf[cursor..cursor + 4] != *session_id {
                    return Ok(None);
                }
                cursor += 4;
                while let Ok((msg, off)) = decode_message(&buf[cursor..]) {
                    println!("decoded message: {:?}", msg);
                    ret.push(msg);
                    cursor += off;
                }
            }
            // if cursor < n - 1 {
            //     let mut i = 0;
            //     while cursor < n - 1 {
            //         buf[i] = buf[cursor];
            //         i += 1;
            //         cursor += 1;
            //     }
            // }
            buf.fill(0);
            Ok(Some(ret))
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
        Err(other) => Err(other.into()),
    }
}
pub fn send_message(sock: &mut TcpStream, msg: Message, session_id: &SessId) -> Result<Option<()>> {
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
/// # Return value
/// None or a Message and cursor offset after decoding it
fn decode_message(bytes: &[u8]) -> Result<(Message, usize)> {
    const MOVED_SZ: usize = 7;
    match bytes[0] {
        0x01 if bytes.len() >= MOVED_SZ => {
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
            Ok((mess, MOVED_SZ))
        }
        0x02 => {
            Ok((Message::Rejected(), 1))
        }
        0x03 => {
            Ok((Message::Accepted(), 1))
        }
        0x04 => {
            Ok((Message::GameDone(), 1))
        }
        _ => Err(anyhow!("invalid message kind")),
    }
}
#[derive(Debug)]
pub enum Message {
    Moved(super::board::BoardMove), // 0x01
    Rejected(), // 0x02
    Accepted(), // 0x03
    GameDone(), // 0x04
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
        Message::GameDone() => {
            bytes.put_u8(0x04);
        }
    }
    bytes.into()
}
