use std::io::{ErrorKind, Read, Write};

use super::game::Game;
use crate::network::client::ClientConnection;
use crate::network::{self, NETBUF_SIZE, SessId};
type Conn = ClientConnection;

pub fn run_client(mut game: Game, addr: &str) {
    let mut client = network::client::ClientConnection::new(addr).unwrap();
    println!("client connecting...");

    game.update_board_data();
    while !game.window_handle.window_should_close() {
        client = match client {
            ClientConnection::Begin(mut tcp, addr, buf) => {
                match tcp.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0x02]) {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            Conn::Begin(tcp, addr, buf)
                        } else {
                            panic!("{e}");
                        }
                    }
                    Ok(_) => ClientConnection::SessionIdRead(tcp, addr, buf),
                }
            }
            ClientConnection::SessionIdRead(mut tcp, addr, mut buf) => {
                match tcp.read(&mut buf[..8]) {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            ClientConnection::SessionIdRead(tcp, addr, buf)
                        } else {
                            panic!("{e}")
                        }
                    }
                    Ok(recieved) => {
                        if recieved != 8 {
                            panic!("handshake length failure")
                        }
                        if buf[..4] != [0xDE, 0xAD, 0xBE, 0xEF] {
                            panic!("handshake header failure")
                        }
                        let id: SessId = buf[4..8].try_into().unwrap();
                        println!("session_id: {:?}", id);
                        ClientConnection::Connected(tcp, addr, id, [0; NETBUF_SIZE], 0)
                    }
                }
            }
            ClientConnection::Connected(mut tcp, a, id, mut buf, cursor) => {
                if let Some(mess) = network::recv_message(&mut tcp, &mut buf, &id).unwrap() {
                    game.recv_mess_queue.push_front(mess);
                }
                while let Some(mess) = game.send_mess_queue.pop_front() {
                    network::send(&mut tcp, mess, &id).unwrap();
                }
                ClientConnection::Connected(tcp, a, id, buf, cursor)
            }
        };
        game.update();
        game.draw();
    }
}
