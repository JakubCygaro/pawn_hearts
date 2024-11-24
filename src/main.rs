mod board;
mod data;
mod helpers;
mod resources;

use board::{BoardPos, MoveBuilder};
use raylib::{
    ffi::{MouseButton, TraceLogLevel},
    math::{Rectangle, Vector2},
    prelude::{self as ray, color::Color, RaylibDraw},
    RaylibHandle, RaylibThread,
};
use resources::*;
use std::{path::PathBuf, str::FromStr};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;
///Margin between the board and window borders
const MARGIN: f32 = 0.1;

fn main() {
    let mut game = GameState::init(WIDTH, HEIGHT);
    game.run()
}

#[derive(Debug)]
struct Selection {
    piece: board::ChessBoardCell,
    taken_from: board::BoardPos,
}

struct GameState {
    board: board::ChessBoard,
    window_handle: RaylibHandle,
    window_thread: RaylibThread,
    width: i32,
    height: i32,
    loader: Box<dyn ResourceLoader>,
    board_data: board::BoardRenderData,
    selected_piece: Option<Selection>,
    reversed: bool,
}
impl GameState {
    pub fn init(width: i32, height: i32) -> Self {
        let (mut window_handle, mut window_thread) = ray::init()
            .width(width)
            .height(height)
            .title("Pawn Hearts")
            .msaa_4x()
            .resizable()
            .log_level(TraceLogLevel::LOG_DEBUG)
            .build();
        window_handle.set_target_fps(60);
        let mut loader = DirectoryResourceLoader::new(PathBuf::from_str("data/").unwrap());
        loader
            .load_all_root(&mut window_handle, &mut window_thread)
            .expect("could not load all textures");
        Self {
            board: board::ChessBoard::new_full(),
            window_handle: window_handle,
            window_thread: window_thread,
            width: width,
            height: height,
            loader: Box::new(loader),
            board_data: board::BoardRenderData::default(),
            selected_piece: None,
            reversed: false,
        }
    }
    pub fn run(&mut self) {
        self.update_board_data();
        while !self.window_handle.window_should_close() {
            self.update();
            self.draw();
        }
    }

    fn update(&mut self) {
        if self.window_handle.is_window_resized() {
            self.resize();
        }
        self.update_mouse();
    }

    fn draw(&mut self) {
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
                    x: self.board_data.rect.width as f32 - point.x,
                    y: self.board_data.rect.height as f32 - point.y,
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
                board::ChessBoardCell::White(_) => {
                    self.selected_piece = Some(Selection {
                        piece: self.board.take_from(pos).unwrap(),
                        taken_from: pos,
                    });
                }
                board::ChessBoardCell::Black(_) => {
                    self.selected_piece = Some(Selection {
                        piece: self.board.take_from(pos).unwrap(),
                        taken_from: pos,
                    });
                }
            }
        }
    }
    fn handle_place(&mut self, pos: BoardPos) {
        if let Some(s) = &self.selected_piece {
            let m = MoveBuilder::new().from(s.taken_from).to(pos).build();
            let selection = self.selected_piece.take().unwrap();
            self.board
                .place_at(selection.taken_from, selection.piece)
                .unwrap();
            self.board.move_piece(m);

            // //place selection in empty cell
            // if let Some(ChessBoardCell::Empty) = self.board.at(pos) {
            //     self.board
            //         .place_at(pos, self.selected_piece.take().unwrap().piece)
            //         .unwrap();
            // } else {
            //     //put it back where it came from if something is there
            //     let selection = self.selected_piece.take().unwrap();
            //     self.board
            //         .place_at(selection.taken_from, selection.piece)
            //         .unwrap();
            // }
        }
    }

    fn resize(&mut self) {
        self.width = self.window_handle.get_screen_width();
        self.height = self.window_handle.get_screen_height();
        self.update_board_data();
    }

    fn update_board_data(&mut self) {
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
}
