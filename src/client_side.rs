use std::io::{ErrorKind, Write};

use super::game::Game;
use crate::network::client::ClientConnection;
use crate::network::{self, MAGIC_N};

pub fn run_client(mut game: Game, addr: &str) {
    let mut client = network::client::ClientConnection::new(addr).unwrap();
    println!("client connecting...");

    game.update_board_data();
    while !game.window_handle.window_should_close() {
        client = match client {
            ClientConnection::Begin(mut tcp, addr, buf) =>
                match tcp.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0x02]){
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        client
                    } else {
                        panic!("{e}");
                    }
                }
                Ok(_) => {
                    ClientConnection::SessionIdReak(tcp, addr, buf)
                }
            },
            ClientConnection::Connected(tcp, a, id, buf, cursor) => {
                println!("connected client, passing...");
                ClientConnection::Connected(tcp, a, id, buf, cursor)
            }
        };
        game.update();
        game.draw();
    }
}

