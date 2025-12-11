use crate::board::{ChessBoardCell, ChessPiece};
use crate::gui::{self, FontWrap};
use crate::network::client::Client;
use crate::network::host::Host;
use crate::network::{Connection, MessageQueue};

use super::board::{self, BoardPos, MoveBuilder};
use super::helpers;
use super::resources::*;
use raylib::{
    ffi::{MouseButton, TraceLogLevel},
    math::{Rectangle, Vector2},
    prelude::{self as ray, color::Color, RaylibDraw},
    RaylibHandle, RaylibThread,
};
use std::net::{IpAddr, SocketAddr};
use std::ops::Not;
use std::{path::PathBuf, str::FromStr};

use super::{board::BoardMove, network::Message};
///
///Margin between the board and window borders
const MARGIN: f32 = 0.1;

#[derive(Debug)]
pub struct Selection {
    piece: board::ChessBoardCell,
    taken_from: board::BoardPos,
}
#[derive(Debug)]
pub struct RunArgs {
    pub address: String,
    pub is_host: bool,
}

pub struct Game {
    board: board::ChessBoard,
    scratch_board: Option<board::ChessBoard>,
    pub window_handle: RaylibHandle,
    pub window_thread: RaylibThread,
    width: i32,
    height: i32,
    loader: Box<dyn ResourceLoader>,
    board_data: board::BoardRenderData,
    selected_piece: Option<Selection>,
    reversed: bool,
    is_host: bool,
    conn: Option<Box<dyn Connection>>,
    send_queue: MessageQueue,
    state: State,
    input_text: String,
    error_msg: Option<String>,
}
#[derive(PartialEq, Clone, Debug)]
enum State {
    Move,
    MovePending(BoardMove),
    WaitReply(BoardMove),
    WaitMove,
    Won,
    Lost,
    SetupConnection,
    ConnectingHost,
    ConnectingClient,
}

pub enum NetworkEvent {
    NotConnected,
    Connecting,
    Connected,
    Broken,
}

impl Game {
    pub fn init(width: i32, height: i32, run_args: Option<RunArgs>) -> Self {
        let (mut window_handle, mut window_thread) = ray::init()
            .width(width)
            .height(height)
            .title("Pawn Hearts")
            .msaa_4x()
            .resizable()
            .log_level(TraceLogLevel::LOG_NONE)
            .build();
        window_handle.set_target_fps(60);
        let mut loader = DirectoryResourceLoader::new(PathBuf::from_str("data/").unwrap());
        loader
            .load_all_root(&mut window_handle, &mut window_thread)
            .expect("could not load all textures");

        let is_host = run_args.as_ref().map(|ra| ra.is_host);

        let conn: Option<Box<dyn Connection>> = match run_args {
            Some(ra) => {
                let c: Box<dyn Connection> = if ra.is_host {
                    Box::new(Host::new(&ra.address).unwrap())
                } else {
                    Box::new(Client::new(&ra.address).unwrap())
                };
                Some(c)
            }
            None => None,
        };

        Self {
            board: board::ChessBoard::new_full(),
            scratch_board: None,
            window_handle,
            window_thread,
            width,
            height,
            loader: Box::new(loader),
            board_data: board::BoardRenderData::default(),
            selected_piece: None,
            reversed: false,
            is_host: true,
            conn,
            state: is_host
                .map(|ih| {
                    if ih {
                        State::ConnectingHost
                    } else {
                        State::ConnectingClient
                    }
                })
                .unwrap_or(State::SetupConnection),
            send_queue: MessageQueue::new(),
            input_text: String::from(""),
            error_msg: None,
        }
    }
    pub fn update(&mut self) {
        if self.window_handle.is_window_resized() {
            self.resize();
        }
        let mut msgs: Vec<Message> = vec![];
        if self.conn.is_some() {
            let conn = self.conn.as_mut().unwrap();
            conn.poll().unwrap();
            while let Some(msg) = conn.recv() {
                println!("recieved message: {:?}", msg);
                msgs.push(msg)
            }
        }
        for m in msgs {
            if let Some(new_state) = self.handle_message(m) {
                self.state = new_state;
            }
        }

        self.state = match self.state {
            State::MovePending(m) => {
                if self.conn.is_some() {
                    self.send_queue.push_back(Message::Moved(m));
                    if self.is_host {
                        State::WaitMove
                    } else {
                        State::WaitReply(m)
                    }
                } else {
                    self.state.clone()
                }
            }
            State::ConnectingClient if self.conn.as_ref().unwrap().is_connected() => {
                self.is_host = false;
                self.reversed = true;
                State::WaitMove
            }
            State::ConnectingHost if self.conn.as_ref().unwrap().is_connected() => State::Move,
            State::Won | State::Lost => {
                println!("Game Finished");
                self.state.clone()
            }
            _ => self.state.clone(),
        };
        self.update_mouse();

        let mut msgs = vec![];
        while let Some(m) = self.send_queue.pop_front() {
            msgs.push(m)
        }
        if self.conn.is_some() {
            let conn = self.conn.as_mut().unwrap();
            for m in msgs {
                conn.send(m);
            }
        }
    }
    fn handle_message(&mut self, msg: Message) -> Option<State> {
        println!("handle message: {:?}", msg);
        if self.is_host {
            self.handle_message_host(msg)
        } else {
            self.handle_message_client(msg)
        }
    }
    fn handle_message_host(&mut self, msg: Message) -> Option<State> {
        match (msg, &self.state) {
            (Message::Moved(m), State::WaitMove) => {
                self.send_queue.push_back(Message::Accepted());
                self.statefull_move_piece(m)
                    .inspect(|_| self.send_queue.push_back(Message::GameDone()))
                    .or(Some(State::Move))
            }
            (Message::Moved(_), _) => {
                self.send_queue.push_back(Message::Rejected());
                Some(State::Move)
            }
            (_, _) => None,
        }
    }
    fn handle_message_client(&mut self, msg: Message) -> Option<State> {
        match (msg, &self.state) {
            (Message::Moved(m), State::WaitMove) => {
                // self.board.move_piece(m);
                // self.send_mess_queue.push_back(Message::GameDone());
                // Some(State::Move)
                self.statefull_move_piece(m)
                    .inspect(|_| self.send_queue.push_back(Message::GameDone()))
                    .or(Some(State::Move))
            }
            (Message::Rejected(), _) => Some(State::WaitMove),
            (Message::Accepted(), State::WaitReply(m)) => {
                // self.board.move_piece(*m);
                // Some(State::WaitMove)
                self.statefull_move_piece(*m)
                    .inspect(|_| self.send_queue.push_back(Message::GameDone()))
                    .or(Some(State::WaitMove))
            }
            (Message::GameDone(), State::WaitReply(m)) => {
                self.statefull_move_piece(*m).or(Some(State::WaitMove))
            }
            (Message::GameDone(), _) => {
                println!("client game done! with state {:?}", self.state);
                None
            }
            _ => None,
        }
    }

    fn update_mouse(&mut self) {
        if self
            .window_handle
            .is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && self.selected_piece.is_none()
            && self.state == State::Move
        {
            if let Some(pos) = self.board_pos() {
                self.handle_select(pos);
            }
        }
        if self
            .window_handle
            .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
            && self.selected_piece.is_some()
        {
            if let Some(pos) = self.board_pos() {
                self.handle_place(pos);
            }
            // let mouse_pos = self.window_handle.get_mouse_position();
            // if let Some(point) = helpers::check_point_on_rect(&self.board_data.rect, mouse_pos) {
            //     let pos = helpers::get_board_pos(&self.board_data, point);
            //     self.handle_place(pos);
            // }
        }
    }

    fn handle_select(&mut self, pos: board::BoardPos) {
        if let Some(_selected) = &self.selected_piece {
            panic!("selecting while other piece was already selected")
        } else {
            let Some(cell) = self.board.at(pos) else {
                return;
            };
            match cell {
                board::ChessBoardCell::Empty => {
                    self.selected_piece = None;
                }
                board::ChessBoardCell::White(_) if self.is_host => {
                    self.selected_piece = Some(Selection {
                        piece: self.board.take_from(pos).unwrap(),
                        taken_from: pos,
                    });
                }
                board::ChessBoardCell::Black(_) if self.is_host.not() => {
                    self.selected_piece = Some(Selection {
                        piece: self.board.take_from(pos).unwrap(),
                        taken_from: pos,
                    });
                }
                _ => self.selected_piece = None,
            }
        }
    }
    fn handle_place(&mut self, pos: BoardPos) {
        if let Some(s) = &self.selected_piece {
            let m = MoveBuilder::new().from(s.taken_from).to(pos).build();
            let selection = self.selected_piece.take().unwrap();
            // put it back for now
            self.board
                .place_at(selection.taken_from, selection.piece)
                .unwrap();

            if !self.is_host {
                // clone the board so that any changes occur only for the copy and dont modify the
                // state of the real board (host does not care and performs their moves on the true
                // board anyways)
                if self.scratch_board.is_none() && !self.is_host {
                    self.scratch_board = self.board.clone().into();
                }
                let scratch: &mut board::ChessBoard = self.scratch_board.as_mut().unwrap();
                if scratch.move_piece(m).is_some() {
                    println!("move pending!");
                    self.state = State::MovePending(m);
                } else {
                    self.scratch_board = None;
                }
            } else if let Some(result) = self.board.move_piece(m) {
                self.send_queue.push_back(Message::Moved(m));
                match is_lost_or_won(self.is_host, &result.pieces_deleted) {
                    Some(EndCheck::Victory) => {
                        self.send_queue.push_back(Message::GameDone());
                        self.state = State::Won
                    }
                    Some(EndCheck::Loss) => {
                        self.send_queue.push_back(Message::GameDone());
                        self.state = State::Lost
                    }
                    _ => self.state = State::WaitMove,
                }
            }
        }
    }

    fn resize(&mut self) {
        self.width = self.window_handle.get_screen_width();
        self.height = self.window_handle.get_screen_height();
        self.update_board_data();
    }

    pub fn update_board_data(&mut self) {
        let center = Vector2 {
            x: self.width as f32 / 2.,
            y: self.height as f32 / 2.,
        };

        let (start, size) = if self.width >= self.height {
            let size = self.height as f32 - self.height as f32 * MARGIN;
            (
                Vector2 {
                    x: center.x - size / 2.,
                    y: center.y - size / 2.,
                },
                size,
            )
        } else {
            let size = self.width as f32 - self.width as f32 * MARGIN;
            (
                Vector2 {
                    x: center.x - size / 2.,
                    y: center.y - size / 2.,
                },
                size,
            )
        };
        let cell_size = size / 8.;

        self.board_data.size = size;
        self.board_data.start = start;
        self.board_data.cell_size = cell_size;
        self.board_data.rect = Rectangle {
            x: start.x,
            y: start.y,
            width: size,
            height: size,
        };
    }
    pub fn on_network_event(&mut self, _ev: NetworkEvent) {}

    /// Takes into consideration wether the move casuses a loss or victory
    /// # Returns
    /// `Some(State::Won | State::Lost)` if the move caused a game ending condition, otherwise
    /// returns None
    fn statefull_move_piece(&mut self, m: BoardMove) -> Option<State> {
        if let Some(res) = self.board.move_piece(m) {
            println!(
                "move_piece result: \ndeleted: {:?}\nset: {:?}\nmoved: {:?}",
                res.pieces_deleted, res.pieces_set, res.pieces_moved
            );
            match is_lost_or_won(self.is_host, &res.pieces_deleted) {
                Some(EndCheck::Victory) => Some(State::Won),
                Some(EndCheck::Loss) => Some(State::Lost),
                _ => None,
            }
        } else {
            None
        }
    }
    pub fn draw(&mut self) {
        let font = self.loader.get_font_no_load("LinLibertine_R.otf").unwrap();
        let fontw = FontWrap::wrap(font.as_ref(), 24., 12.);
        match self.state {
            State::SetupConnection => {
                self.draw_setup_connection();
            }
            State::ConnectingClient if !self.conn.as_ref().unwrap().is_connected() => {
                let mut draw_handle = self.window_handle.begin_drawing(&self.window_thread);
                draw_handle.clear_background(Color::WHITESMOKE);
                gui::text(
                    &mut draw_handle,
                    Vector2 {
                        x: (self.width as f32 / 2.),
                        y: (self.height as f32 / 2.),
                    },
                    "Connecting to host",
                    fontw,
                );
            }
            State::ConnectingHost if !self.conn.as_ref().unwrap().is_connected() => {
                let mut draw_handle = self.window_handle.begin_drawing(&self.window_thread);
                draw_handle.clear_background(Color::WHITESMOKE);
                gui::text(
                    &mut draw_handle,
                    Vector2 {
                        x: (self.width as f32 / 2.),
                        y: (self.height as f32 / 2.),
                    },
                    "Waiting for client",
                    fontw,
                );
            }
            State::Won | State::Lost => {
                self.draw_board();
                let mut draw_handle = self.window_handle.begin_drawing(&self.window_thread);
                let pos = Vector2 {
                    x: (self.width as f32 / 2.),
                    y: (self.height as f32 / 2.),
                };
                let sz = Vector2 {
                    x: (self.width as f32 / 4.),
                    y: (self.height as f32 / 8.),
                };
                draw_handle.draw_rectangle_v(pos - (sz / 2.), sz, Color::GRAY);
                let msg = match self.state {
                    State::Won => "You won",
                    State::Lost => "You lost",
                    _ => unreachable!(),
                };
                gui::text(&mut draw_handle, pos, msg, fontw);
            }
            _ => {
                self.draw_board();
            }
        }
    }
    fn draw_setup_connection(&mut self) {
        let mut draw_handle = self.window_handle.begin_drawing(&self.window_thread);
        draw_handle.clear_background(Color::WHITESMOKE);
        let font = self.loader.get_font_no_load("LinLibertine_R.otf").unwrap();
        let fontw = FontWrap::wrap(font.as_ref(), 24., 12.);
        let connect_pos = Vector2 {
            x: (self.width as f32 / 2.),
            y: (self.height as f32 / 2.),
        };
        let (client, sz) = gui::button(&mut draw_handle, connect_pos, "Connect", fontw);
        let (host, host_sz) = gui::button(
            &mut draw_handle,
            Vector2 {
                x: connect_pos.x,
                y: connect_pos.y + (sz.y * 1.5),
            },
            "Host",
            fontw,
        );
        let (input, _) = gui::text_input(
            &mut draw_handle,
            Vector2 {
                x: connect_pos.x,
                y: connect_pos.y - (sz.y * 1.5),
            },
            &mut self.input_text,
            fontw,
        );
        if !input && self.error_msg.is_some() {
            gui::text(
                &mut draw_handle,
                Vector2 {
                    x: connect_pos.x,
                    y: connect_pos.y + (sz.y * 1.5) + (host_sz.y * 1.5),
                },
                self.error_msg.as_ref().unwrap(),
                fontw,
            );
        } else {
            self.error_msg = None;
        }
        match (client, host, SocketAddr::from_str(self.input_text.as_str())) {
            (true, false, Ok(addr)) => {
                self.state = State::ConnectingClient;
                self.conn = Some(Box::new(Client::new(&addr.to_string()).unwrap()))
            }
            (false, true, Ok(addr)) => {
                self.state = State::ConnectingHost;
                self.conn = Some(Box::new(Host::new(&addr.to_string()).unwrap()))
            }
            (false, false, _) => (),
            (_, _, Err(e)) => {
                self.error_msg = "Invalid ip address".to_owned().into();
            }
            _ => (),
        };
    }
    fn draw_board(&mut self) {
        let mut draw_handle = self.window_handle.begin_drawing(&self.window_thread);
        draw_handle.clear_background(Color::BLACK);

        let mut color = Color::WHITESMOKE;

        let iter = if !self.reversed {
            self.board.cells().iter().enumerate().collect::<Vec<_>>()
        } else {
            self.board.cells().iter().rev().enumerate().collect()
        };

        for (n, cell) in iter {
            let col = n % 8;
            let row = n / 8;
            let rect = Rectangle {
                x: self.board_data.start.x + col as f32 * self.board_data.cell_size,
                y: self.board_data.start.y + row as f32 * self.board_data.cell_size,
                width: self.board_data.cell_size,
                height: self.board_data.cell_size,
            };
            draw_handle.draw_rectangle_pro(rect, Vector2::zero(), 0., color);
            color = if col == 7 {
                color
            } else if color == Color::WHITESMOKE {
                Color::BURLYWOOD
            } else {
                Color::WHITESMOKE
            };

            if let Some(texture) = cell.get_texture_path() {
                if let Some(texture) = self.loader.get_texture_no_load(texture) {
                    let source = Rectangle {
                        height: texture.height as f32,
                        width: texture.width as f32,
                        x: 0.,
                        y: 0.,
                    };
                    draw_handle.draw_texture_pro(
                        texture.as_ref(),
                        source,
                        rect,
                        Vector2::zero(),
                        0.,
                        Color::WHITE,
                    )
                }
            }
        }
        if let Some(selection) = &self.selected_piece {
            let mouse = draw_handle.get_mouse_position();
            let cell_sz = self.board_data.cell_size;
            let cell_pos = Vector2 {
                x: mouse.x - (cell_sz / 2.),
                y: mouse.y - (cell_sz / 2.),
            };
            let texture = self
                .loader
                .get_texture_no_load(
                    selection
                        .piece
                        .get_texture_path()
                        .expect("selection with no texture"),
                )
                .expect("texture for selection missing");
            draw_handle.draw_texture_pro(
                texture.as_ref(),
                Rectangle {
                    height: texture.height as f32,
                    width: texture.width as f32,
                    x: 0.,
                    y: 0.,
                },
                Rectangle {
                    height: cell_sz,
                    width: cell_sz,
                    x: cell_pos.x,
                    y: cell_pos.y,
                },
                Vector2::zero(),
                0.,
                Color::WHITE,
            )
        }
    }
    fn board_pos(&self) -> Option<BoardPos> {
        let mouse_pos = self.window_handle.get_mouse_position();
        if let Some(point) = helpers::check_point_on_rect(&self.board_data.rect, mouse_pos) {
            let point = if self.reversed {
                Vector2 {
                    x: self.board_data.rect.width - point.x,
                    y: self.board_data.rect.height - point.y,
                }
            } else {
                point
            };
            let pos = helpers::get_board_pos(&self.board_data, point);
            Some(pos)
        } else {
            None
        }
    }
}

enum EndCheck {
    Loss,
    Victory,
}
fn is_lost_or_won(is_host: bool, deleted: &Vec<ChessBoardCell>) -> Option<EndCheck> {
    for cell in deleted {
        println!("deleted: {:?}", deleted);
        match *cell {
            ChessBoardCell::Black(ChessPiece::King(_)) if is_host => {
                return Some(EndCheck::Victory)
            }
            ChessBoardCell::White(ChessPiece::King(_)) if is_host => return Some(EndCheck::Loss),
            ChessBoardCell::Black(ChessPiece::King(_)) => return Some(EndCheck::Loss),
            ChessBoardCell::White(ChessPiece::King(_)) => return Some(EndCheck::Victory),
            _ => continue,
        };
    }
    None
}
