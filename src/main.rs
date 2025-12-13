use std::process::exit;

use self::game::{Game, RunArgs};
mod gui;
pub mod board;
pub mod data;
pub mod game;
pub mod helpers;
pub mod network;
pub mod resources;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let args = if args.len() > 1 {
        if args.len() != 3 {
            eprintln!("improper argument count");
            exit(-1)
        }
        let address = args[1].clone();
        let is_host = args[2].parse::<bool>().unwrap();
        Some(RunArgs { address, is_host })
    } else {
        None
    };
    let mut game = Game::init(WIDTH, HEIGHT, args);
    game.update_board_data();
    while !game.should_close() {
        game.update();
        game.draw();
    }
}
