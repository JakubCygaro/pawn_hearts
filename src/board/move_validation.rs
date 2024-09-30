use super::{BoardMove, BoardPos, ChessBoard, ChessBoardCell, ChessPiece};
use lazy_static::lazy_static;
use std::collections::HashMap;

type MoveChecker = fn(BoardMove, &ChessBoard) -> bool;
type Move = BoardMove;
type Board = ChessBoard;
type Cell = ChessBoardCell;
type Piece = ChessPiece;
lazy_static! {
    pub static ref MOVEMAP: HashMap<ChessBoardCell, MoveChecker> = {
        HashMap::from([
            (Cell::Black(Piece::Pawn), black_pawn as MoveChecker),
            (Cell::White(Piece::Pawn), white_pawn as MoveChecker),
            (Cell::Black(Piece::Bishop), bishop as MoveChecker),
            (Cell::White(Piece::Bishop), bishop as MoveChecker),
            (Cell::Black(Piece::Rook), rook as MoveChecker),
            (Cell::White(Piece::Rook), rook as MoveChecker),
            (Cell::Black(Piece::Knight), knight as MoveChecker),
            (Cell::White(Piece::Knight), knight as MoveChecker),
            (Cell::Black(Piece::Queen), queen as MoveChecker),
            (Cell::White(Piece::Queen), queen as MoveChecker),
            (Cell::Black(Piece::King), king as MoveChecker),
            (Cell::White(Piece::King), king as MoveChecker),
        ])
    };
}

fn black_pawn(mv: BoardMove, b: &ChessBoard) -> bool {
    match mv {
        BoardMove {
            rows: 1,
            columns: -1,
            ..
        }
        | BoardMove {
            rows: 1,
            columns: 1,
            ..
        } => {
            if let Some(&ChessBoardCell::White(_)) = b.at(mv.to) {
                true
            } else {
                false
            }
        }
        BoardMove { rows: 1, .. } =>
        //| BoardMove { rows: 2, .. } if mv.from.row == 1
        {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn white_pawn(mv: BoardMove, b: &ChessBoard) -> bool {
    match mv {
        BoardMove {
            rows: -1,
            columns: -1,
            ..
        }
        | BoardMove {
            rows: -1,
            columns: 1,
            ..
        } => {
            if let Some(&ChessBoardCell::Black(_)) = b.at(mv.to) {
                true
            } else {
                false
            }
        }
        BoardMove { rows: -1, .. } => {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn bishop(mv: Move, b: &Board) -> bool {
    if mv.columns.abs() == mv.rows.abs() {
        for (r, c) in bisex_range(0, mv.rows)
            .skip(1)
            .zip(bisex_range(0, mv.columns).skip(1))
        {
            let to_check = BoardPos {
                row: (mv.from.row as isize + r) as usize,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return false;
            };
        }
        true
    } else {
        false
    }
}

fn rook(mv: Move, b: &Board) -> bool {
    if mv.columns == 0 {
        for r in bisex_range(0, mv.rows).skip(1) {
            let to_check = BoardPos {
                row: (mv.from.row as isize + r) as usize,
                col: mv.from.col,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return false;
            };
        }
        return true;
    } else if mv.rows == 0 {
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return false;
            };
        }
        return true;
    } else {
        false
    }
}

fn knight(mv: Move, _: &Board) -> bool {
    (mv.columns.abs() == 2 && mv.rows.abs() == 1) || (mv.columns.abs() == 1 && mv.rows.abs() == 2)
}

fn queen(mv: Move, b: &Board) -> bool {
    bishop(mv, b) || rook(mv, b)
}

fn king(mv: Move, _: &Board) -> bool {
    mv.columns.abs() <= 1 && mv.rows.abs() <= 1
}

fn bisex_range(a: isize, b: isize) -> impl Iterator<Item = isize> {
    let mut start = a;
    let end = b;
    std::iter::from_fn(move || {
        use std::cmp::Ordering::*;
        match start.cmp(&end) {
            Less => {
                start += 1;
                Some(start - 1)
            }
            Equal => None,
            Greater => {
                start -= 1;
                Some(start + 1)
            }
        }
    })
}
