use std::io::{ErrorKind, Read, Write};

use crate::network::{self, MAGIC_N};
use crate::network::host::HostConnection;

use super::game::Game;
pub fn run_host(mut game: Game, addr: &str) {
    let mut host = network::host::HostConnection::new(addr).unwrap();
    println!("connecting...");

    game.update_board_data();
    while !game.window_handle.window_should_close() {
        host = match host {
            HostConnection::Begin(ref list, id, buf) => {
                match list.accept() {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            host
                        } else {
                            panic!("{e}");
                        }
                    }
                    Ok((recv, peer)) => HostConnection::HandshakeRead(recv, peer, id, buf),
                }
            }
            HostConnection::HandshakeRead(mut tcp, addr, id, mut buf) => {
                match tcp.read(&mut buf[..8]) {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            HostConnection::HandshakeRead(tcp, addr, id, buf)
                        } else {
                            panic!("{e}")
                        }
                    }
                    Ok(recieved) => {
                        if recieved != 5 {
                            panic!("handshake length was improper");
                        }
                        if buf[0..4] != MAGIC_N{
                            panic!("handshake failed");
                        }
                        if buf[4..5] != [0x02] {
                            panic!("handshake failed");
                        }
                        HostConnection::HandshakeRespond(tcp, addr, id, buf)
                    },
                }
            },
            HostConnection::HandshakeRespond(mut tcp, addr, id, mut buf) => {
                buf[..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
                buf[4..8].copy_from_slice(&id);
                println!("sending session_id: {:?}", &[..8]);
                let res = tcp.write_all(&buf[..8]);
                match res {
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            HostConnection::HandshakeRespond(tcp, addr, id, [0; 128])
                        } else {
                            panic!("{e}")
                        }
                    },
                    Ok(_) => {
                        HostConnection::Connected(tcp, addr, id, [0; 128], 0)
                    }
                }
            }
            _ => todo!()
        };
        game.update();
        game.draw();
    }
}
