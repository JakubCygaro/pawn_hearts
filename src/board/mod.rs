mod move_validation;

use anyhow::anyhow;
use bytes::BufMut;
use move_validation::{SideEffect, ValidationResult};
use raylib::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct BoardMove {
    from: BoardPos,
    to: BoardPos,
    rows: isize,
    columns: isize,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoardMoveResult {
    pub pieces_deleted: Vec<ChessBoardCell>,
    pub pieces_moved: Vec<(ChessBoardCell, BoardMove)>,
    pub pieces_set: Vec<(ChessBoardCell, BoardPos)>,
}

impl BoardMove {
    pub fn new(from: BoardPos, to: BoardPos) -> Self {
        let rows = to.row as isize - from.row as isize;
        let columns = to.col as isize - from.col as isize;

        Self {
            from,
            to,
            rows,
            columns,
        }
    }
    pub fn to_bytes(&self) -> bytes::Bytes {
        let mut bytes = bytes::BytesMut::with_capacity(6);
        bytes.put_u8(self.from.row as u8);
        bytes.put_u8(self.from.col as u8);
        bytes.put_u8(self.to.row as u8);
        bytes.put_u8(self.to.col as u8);
        bytes.put_u8(self.rows as u8);
        bytes.put_u8(self.columns as u8);
        bytes.into()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct MoveBuilder {
    from: BoardPos,
    to: BoardPos,
    rows: isize,
    columns: isize,
}

impl MoveBuilder {
    pub fn new() -> Self {
        Self {
            from: BoardPos::default(),
            to: BoardPos::default(),
            rows: isize::default(),
            columns: isize::default(),
        }
    }

    pub fn from(&mut self, pos: BoardPos) -> &mut Self {
        self.from = pos;
        self
    }

    pub fn to(&mut self, pos: BoardPos) -> &mut Self {
        self.to = pos;
        self
    }

    pub fn rows(&mut self, r: isize) -> &mut Self {
        self.rows = r;
        self.to = BoardPos {
            col: self.to.col,
            row: self.from.row + r as usize,
        };
        self
    }

    pub fn columns(&mut self, c: isize) -> &mut Self {
        self.columns = c;
        self.to = BoardPos {
            col: self.from.col + c as usize,
            row: self.to.row,
        };
        self
    }

    pub fn build(self) -> BoardMove {
        BoardMove::new(self.from, self.to)
    }
}

/// Board row and column
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BoardPos {
    pub row: usize,
    pub col: usize,
}

impl BoardPos {
    fn to_index(self) -> usize {
        (self.row * 8) + self.col
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChessBoardCell {
    White(ChessPiece),
    Black(ChessPiece),
    Empty,
}

impl ChessBoardCell {
    /// extracts the piece from the cell
    /// # Returns
    /// `Some` if the cell contained a piece, otherwise `None`
    pub fn take_piece(self) -> Option<ChessPiece> {
        match self {
            Self::White(piece) => Some(piece),
            Self::Black(piece) => Some(piece),
            _ => None,
        }
    }
}

impl ChessBoardCell {
    pub fn get_texture_path(&self) -> Option<&'static str> {
        match self {
            ChessBoardCell::Black(p) => match p {
                ChessPiece::Bishop => Some("bishop_black.png"),
                ChessPiece::King(_) => Some("king_black.png"),
                ChessPiece::Knight => Some("knight_black.png"),
                ChessPiece::Pawn(_) => Some("pawn_black.png"),
                ChessPiece::Queen => Some("queen_black.png"),
                ChessPiece::Rook => Some("rook_black.png"),
            },
            ChessBoardCell::White(p) => match p {
                ChessPiece::Bishop => Some("bishop_white.png"),
                ChessPiece::King(_) => Some("king_white.png"),
                ChessPiece::Knight => Some("knight_white.png"),
                ChessPiece::Pawn(_) => Some("pawn_white.png"),
                ChessPiece::Queen => Some("queen_white.png"),
                ChessPiece::Rook => Some("rook_white.png"),
            },
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LongStart {
    Before,
    RightNow,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChessPiece {
    Pawn(LongStart),
    Bishop,
    Knight,
    Rook,
    Queen,
    King(bool),
}

pub struct BoardRenderData {
    pub start: Vector2,
    pub size: f32,
    pub cell_size: f32,
    pub rect: Rectangle,
}
impl Default for BoardRenderData {
    fn default() -> Self {
        Self {
            start: Vector2::zero(),
            size: 0.,
            cell_size: 0.,
            rect: Rectangle::default(),
        }
    }
}

macro_rules! p {
    (BP) => {
        ChessBoardCell::Black(ChessPiece::Pawn(LongStart::Before))
    };
    (BR) => {
        ChessBoardCell::Black(ChessPiece::Rook)
    };
    (BK) => {
        ChessBoardCell::Black(ChessPiece::Knight)
    };
    (BB) => {
        ChessBoardCell::Black(ChessPiece::Bishop)
    };
    (BQ) => {
        ChessBoardCell::Black(ChessPiece::Queen)
    };
    (BKI) => {
        ChessBoardCell::Black(ChessPiece::King(false))
    };
    (E) => {
        ChessBoardCell::Empty
    };
    (WP) => {
        ChessBoardCell::White(ChessPiece::Pawn(LongStart::Before))
    };
    (WR) => {
        ChessBoardCell::White(ChessPiece::Rook)
    };
    (WK) => {
        ChessBoardCell::White(ChessPiece::Knight)
    };
    (WB) => {
        ChessBoardCell::White(ChessPiece::Bishop)
    };
    (WQ) => {
        ChessBoardCell::White(ChessPiece::Queen)
    };
    (WKI) => {
        ChessBoardCell::White(ChessPiece::King(false))
    };
}

#[derive(Debug, Clone)]
pub struct ChessBoard {
    cells: Vec<ChessBoardCell>,
}

impl ChessBoard {
    pub fn at(&self, pos: BoardPos) -> Option<&ChessBoardCell> {
        if pos.row > 8 || pos.col > 8 {
            None
        } else {
            self.cells.get(pos.to_index())
        }
    }
    pub fn take_from(&mut self, pos: BoardPos) -> Option<ChessBoardCell> {
        let (row, col) = (pos.row, pos.col);
        if row > 8 || col > 8 {
            None
        } else {
            let index = pos.to_index();
            let cell = self.cells.get(index).unwrap().to_owned();
            self.cells[index] = ChessBoardCell::Empty;

            Some(cell)
        }
    }
    pub fn place_at(&mut self, pos: BoardPos, cell: ChessBoardCell) -> anyhow::Result<()> {
        let (row, col) = (pos.row, pos.col);
        if row > 8 || col > 8 {
            Err(anyhow!("position out of bounds"))
        } else {
            let index = pos.to_index();
            self.cells[index] = cell;
            Ok(())
        }
    }
    /// returns Some when the move passed validation
    /// and thus was executed
    pub fn move_piece(&mut self, m: BoardMove) -> Option<BoardMoveResult> {
        let ValidationResult::Valid(Some(side_effects)) = self.validate_move(m) else {
            return None;
        };
        let mut res = BoardMoveResult {
            pieces_deleted: vec![],
            pieces_moved: vec![],
            pieces_set: vec![],
        };
        for side_effect in side_effects.into_iter().rev() {
            match side_effect {
                SideEffect::Delete(p, c) => {
                    res.pieces_deleted.push(c);
                }
                SideEffect::Move(m) => {
                    let piece = self.take_from(m.from).unwrap();
                    self.place_at(m.to, piece).unwrap();
                    res.pieces_moved.push((piece, m));
                }
                SideEffect::SetAt(p, piece) => {
                    self.place_at(p, piece).unwrap();
                    res.pieces_set.push((piece, p));
                }
            }
        }
        Some(res)
    }

    fn validate_move(&self, m: BoardMove) -> ValidationResult {
        if m.to == m.from {
            return ValidationResult::NotValid;
        }
        let from_cell = self.at(m.from);
        let Some(from_cell) = from_cell else {
            return ValidationResult::NotValid;
        };
        if *from_cell == ChessBoardCell::Empty {
            return ValidationResult::NotValid;
        }

        if let Some(at_cell) = self.at(m.to) {
            //check if the target piece is not of the same colour as the from piece
            if !matches!(
                (from_cell, at_cell),
                (_, ChessBoardCell::Empty)
                    | (ChessBoardCell::Black(_), ChessBoardCell::White(_))
                    | (ChessBoardCell::White(_), ChessBoardCell::Black(_))
            ) {
                ValidationResult::NotValid
            } else if move_validation::MOVEMAP.contains_key(from_cell) {
                let res = move_validation::MOVEMAP[from_cell](m, self);
                match res {
                    ValidationResult::Valid(se) => {
                        let se = se
                            .map(|mut s| {
                                s.push(SideEffect::Move(m));
                                s
                            })
                            .or(Some(vec![SideEffect::Move(m)]))
                            .map(|mut s| {
                                if !matches!(at_cell, ChessBoardCell::Empty) {
                                    s.push(SideEffect::Delete(m.to, *at_cell))
                                }
                                s
                            });
                        ValidationResult::Valid(se)
                    }
                    // ValidationResult::Valid(Some(mut se)) => {
                    //
                    //
                    //
                    //     se.push(SideEffect::Move(m));
                    //     ValidationResult::Valid(Some(se))
                    // }
                    // ValidationResult::Valid(None) => {
                    //     let se = vec![SideEffect::Move(m)];
                    //     ValidationResult::Valid(Some(se))
                    // }
                    ValidationResult::NotValid => ValidationResult::NotValid,
                }
            } else {
                ValidationResult::NotValid
            }
        } else {
            ValidationResult::NotValid
        }
    }
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl ChessBoard {
    pub fn new_empty() -> Self {
        Self {
            cells: vec![ChessBoardCell::Empty; 8 * 8],
        }
    }
    pub fn new_full() -> Self {
        Self {
            cells: vec![
                p!(BR),
                p!(BK),
                p!(BB),
                p!(BQ),
                p!(BKI),
                p!(BB),
                p!(BK),
                p!(BR),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(BP),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(E),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WP),
                p!(WR),
                p!(WK),
                p!(WB),
                p!(WQ),
                p!(WKI),
                p!(WB),
                p!(WK),
                p!(WR),
            ],
        }
    }
    pub fn cells(&self) -> &Vec<ChessBoardCell> {
        &self.cells
    }
}
