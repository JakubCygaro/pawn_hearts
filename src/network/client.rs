use super::NetworkConnection;
use anyhow::{anyhow, Result};
use std::net::UdpSocket;
use std::sync::mpsc;

pub struct ClientConnection {
    read_handle: mpsc::Receiver<super::NetworkOutcome>,
    session_id: Option<[u8; 4]>,
}
impl ClientConnection {
    pub fn create(host_adress: &str) -> Result<Self> {
        let conn = UdpSocket::bind("127.0.0.1:0")?;
        println!("client bound to : {}", conn.local_addr()?);
        conn.connect(host_adress);
        println!("client connected to : {}", conn.peer_addr()?);
        let mut buf = [0u8; 8];
        conn.send(&[0xDE, 0xAD, 0xBE, 0xEF, 0x02])?; 
        let recieved = conn.recv(&mut buf)?;
        if recieved != 8 {
            return Err(anyhow!("handshake length failure"));
        }
        if buf[..4] != [0xDE, 0xAD, 0xBE, 0xEF] {
            return Err(anyhow!("handshake header failure"));
        }
        let session_id: [u8; 4] = buf[4..8].try_into()?;
        println!("session_id: {:?}", session_id);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move ||{
           connection_loop(tx); 
        });
        Ok(Self {
            read_handle: rx,
            session_id: None,
        })
    }
}
fn connection_loop(tx: mpsc::Sender<super::NetworkOutcome>) {

}

