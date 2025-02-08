use super::NetworkConnection;
use anyhow::{anyhow, Result};
use std::net::{UdpSocket};
use std::sync::mpsc;
pub struct HostConnection {
    read_handle: mpsc::Receiver<super::NetworkOutcome>,
    session_id: [u8; 4],
}

impl HostConnection {
    pub fn create(address: &str) -> Result<Self> {
        let conn = UdpSocket::bind(address)?;
        let session_id: [u8; 4] = rand::random();
        let mut buf = [0u8; 8];
        let (recieved, peer) = conn.recv_from(&mut buf)?;
        if recieved != 5 {
            return Err(anyhow!("handshake length was improper"));
        }
        if buf[0..5] != [0xDE, 0xAD, 0xBE, 0xEF, 0x02] {
            return Err(anyhow!("handshake failed"));
        }
        conn.connect(peer)?;
        buf[..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        buf[4..].copy_from_slice(&session_id);
        println!("sending session_id: {:?}", buf);
        conn.send(&buf)?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move ||{
           connection_loop(tx); 
        });
        Ok(Self {
            read_handle: rx,
            session_id: rand::random(),
        })
    }
}

fn connection_loop(tx: mpsc::Sender<super::NetworkOutcome>) {

}
