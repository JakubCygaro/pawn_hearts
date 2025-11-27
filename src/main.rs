pub mod board;
pub mod data;
pub mod helpers;
pub mod network;
pub mod resources;
pub mod host_side;
pub mod client_side;
pub mod game;


const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let adress = args[1].clone();
    let host = args[2].parse::<bool>().unwrap();
    if host {
        host_side::run_host(game::Game::init(WIDTH, HEIGHT), &adress)
    } else {
        client_side::run_client(game::Game::init(WIDTH, HEIGHT), &adress)
    }
}

