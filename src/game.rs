use crate::board::{ChessBoardCell, ChessPiece};
use crate::network::MessageQueue;

use super::board::{self, BoardPos, MoveBuilder};
use super::helpers;
use super::network;
use super::resources::*;
use raylib::{
    ffi::{MouseButton, TraceLogLevel},
    math::{Rectangle, Vector2},
    prelude::{self as ray, color::Color, RaylibDraw},
    RaylibHandle, RaylibThread,
};
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
    pub recv_mess_queue: network::MessageQueue,
    pub send_mess_queue: network::MessageQueue,
    state: State,
}
#[derive(PartialEq, Clone, Debug)]
enum State {
    Move,
    MovePending(BoardMove),
    WaitReply(BoardMove),
    WaitMove,
    Won,
    Lost,
}

pub enum NetworkEvent {
    NotConnected,
    Connecting,
    Connected,
    Broken,
}

impl Game {
    pub fn init(width: i32, height: i32, is_host: bool) -> Self {
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
            reversed: !is_host,
            is_host,
            recv_mess_queue: MessageQueue::new(),
            send_mess_queue: MessageQueue::new(),
            state: if is_host {
                State::Move
            } else {
                State::WaitMove
            },
        }
    }
    pub fn update(&mut self) {
        if self.window_handle.is_window_resized() {
            self.resize();
        }
        while let Some(msg) = self.recv_mess_queue.pop_front() {
            if let Some(new_state) = self.handle_message(msg) {
                self.state = new_state;
            }
        }
        self.state = match self.state {
            State::MovePending(m) => {
                self.send_mess_queue.push_back(Message::Moved(m));
                if self.is_host {
                    State::WaitMove
                } else {
                    State::WaitReply(m)
                }
            }
            _ => self.state.clone(),
        };
        self.update_mouse();
        println!("{} state is: {:?}", if self.is_host { "Host" } else { "Client" }, self.state);
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
                self.send_mess_queue.push_back(Message::Accepted());
                self.statefull_move_piece(m)
                    .inspect(|_| self.send_mess_queue.push_back(Message::GameDone()))
                    .or(Some(State::Move))
            }
            (Message::Moved(_), _) => {
                self.send_mess_queue.push_back(Message::Rejected());
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
                    .inspect(|_| self.send_mess_queue.push_back(Message::GameDone()))
                    .or(Some(State::Move))
            }
            (Message::Rejected(), _) => Some(State::WaitMove),
            (Message::Accepted(), State::WaitReply(m)) => {
                // self.board.move_piece(*m);
                // Some(State::WaitMove)
                self.statefull_move_piece(*m)
                    .inspect(|_| self.send_mess_queue.push_back(Message::GameDone()))
                    .or(Some(State::WaitMove))
            }
            _ => None,
        }
    }

    pub fn draw(&mut self) {
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
        if let Some(selected) = &self.selected_piece {
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
                self.send_mess_queue.push_back(Message::Moved(m));
                match is_lost_or_won(self.is_host, &result.pieces_deleted) {
                    Some(EndCheck::Victory) => {
                        self.send_mess_queue.push_back(Message::GameDone());
                        self.state = State::Won
                    }
                    Some(EndCheck::Loss) => {
                        self.send_mess_queue.push_back(Message::GameDone());
                        self.state = State::Lost
                    }
                    _ => { self.state = State::WaitMove }
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
    pub fn on_network_event(&mut self, ev: NetworkEvent) {}

    /// Takes into consideration wether the move casuses a loss or victory
    /// # Returns
    /// `Some(State::Won | State::Lost)` if the move caused a game ending condition, otherwise
    /// returns None
    fn statefull_move_piece(&mut self, m: BoardMove) -> Option<State> {
        if let Some(res) = self.board.move_piece(m) {
            match is_lost_or_won(self.is_host, &res.pieces_deleted) {
                Some(EndCheck::Victory) => Some(State::Won),
                Some(EndCheck::Loss) => Some(State::Lost),
                _ => None,
            }
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
