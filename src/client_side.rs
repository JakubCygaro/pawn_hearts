use super::game::Game;
pub fn run_client(mut game: Game, addr: &str) {
    println!("connecting...");
    //self.network_conn.connect().unwrap();
    game.update_board_data();
    while !game.window_handle.window_should_close() {
        game.update();
        game.draw();
    }
}

