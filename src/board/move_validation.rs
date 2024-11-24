use super::{BoardMove, BoardPos, ChessBoard, ChessBoardCell, ChessPiece, LongStart};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub enum SideEffect {
    Delete(BoardPos),
    Move(BoardMove),
    SetAt(BoardPos, ChessBoardCell),
}
pub enum ValidationResult {
    Valid(Option<Vec<SideEffect>>),
    NotValid,
}

type MoveChecker = fn(BoardMove, &ChessBoard) -> ValidationResult;
type Move = BoardMove;
type Board = ChessBoard;
type Cell = ChessBoardCell;
type Piece = ChessPiece;
lazy_static! {
    pub static ref MOVEMAP: HashMap<ChessBoardCell, MoveChecker> = {
        HashMap::from([
            (
                Cell::Black(Piece::Pawn(LongStart::Before)),
                black_pawn as MoveChecker,
            ),
            (
                Cell::White(Piece::Pawn(LongStart::Before)),
                white_pawn as MoveChecker,
            ),
            (
                Cell::Black(Piece::Pawn(LongStart::RightNow)),
                black_pawn as MoveChecker,
            ),
            (
                Cell::White(Piece::Pawn(LongStart::RightNow)),
                white_pawn as MoveChecker,
            ),
            (
                Cell::Black(Piece::Pawn(LongStart::After)),
                black_pawn as MoveChecker,
            ),
            (
                Cell::White(Piece::Pawn(LongStart::After)),
                white_pawn as MoveChecker,
            ),
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

fn black_pawn(mv: BoardMove, b: &ChessBoard) -> ValidationResult {
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
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::Black(ChessPiece::Pawn(LongStart::After)),
                )]))
            } else if let Some(&ChessBoardCell::White(ChessPiece::Pawn(LongStart::RightNow))) = b
                .at(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col - 1,
                })
            {
                ValidationResult::Valid(Some(vec![SideEffect::Delete(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col - 1,
                })]))
            } else if let Some(&ChessBoardCell::White(ChessPiece::Pawn(LongStart::RightNow))) = b
                .at(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col + 1,
                })
            {
                ValidationResult::Valid(Some(vec![SideEffect::Delete(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col + 1,
                })]))
            } else {
                ValidationResult::NotValid
            }
        }
        BoardMove { rows: 2, .. } if mv.from.row == 1 => {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::Black(ChessPiece::Pawn(LongStart::RightNow)),
                )]))
            } else {
                ValidationResult::NotValid
            }
        }
        BoardMove { rows: 1, .. } => {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::Black(ChessPiece::Pawn(LongStart::After)),
                )]))
            } else {
                ValidationResult::NotValid
            }
        }
        _ => ValidationResult::NotValid,
    }
}

fn white_pawn(mv: BoardMove, b: &ChessBoard) -> ValidationResult {
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
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::White(ChessPiece::Pawn(LongStart::After)),
                )]))
            } else if let Some(&ChessBoardCell::Black(ChessPiece::Pawn(LongStart::RightNow))) = b
                .at(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col - 1,
                })
            {
                ValidationResult::Valid(Some(vec![SideEffect::Delete(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col - 1,
                })]))
            } else if let Some(&ChessBoardCell::Black(ChessPiece::Pawn(LongStart::RightNow))) = b
                .at(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col + 1,
                })
            {
                ValidationResult::Valid(Some(vec![SideEffect::Delete(BoardPos {
                    row: mv.from.row,
                    col: mv.from.col + 1,
                })]))
            } else {
                ValidationResult::NotValid
            }
        }
        BoardMove { rows: -2, .. } if mv.from.row == 6 => {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::White(ChessPiece::Pawn(LongStart::RightNow)),
                )]))
            } else {
                ValidationResult::NotValid
            }
        }
        BoardMove { rows: -1, .. } => {
            if let Some(&ChessBoardCell::Empty) = b.at(mv.to) {
                ValidationResult::Valid(Some(vec![SideEffect::SetAt(
                    mv.to,
                    ChessBoardCell::White(ChessPiece::Pawn(LongStart::After)),
                )]))
            } else {
                ValidationResult::NotValid
            }
        }
        _ => ValidationResult::NotValid,
    }
}

fn bishop(mv: Move, b: &Board) -> ValidationResult {
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
                return ValidationResult::NotValid;
            };
        }
        ValidationResult::Valid(None)
    } else {
        ValidationResult::NotValid
    }
}

fn rook(mv: Move, b: &Board) -> ValidationResult {
    if mv.columns == 0 {
        for r in bisex_range(0, mv.rows).skip(1) {
            let to_check = BoardPos {
                row: (mv.from.row as isize + r) as usize,
                col: mv.from.col,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return ValidationResult::NotValid;
            };
        }
        return ValidationResult::Valid(None);
    } else if mv.rows == 0 {
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return ValidationResult::NotValid;
            };
        }
        return ValidationResult::Valid(None);
    } else {
        ValidationResult::NotValid
    }
}

fn knight(mv: Move, _: &Board) -> ValidationResult {
    if (mv.columns.abs() == 2 && mv.rows.abs() == 1)
        || (mv.columns.abs() == 1 && mv.rows.abs() == 2)
    {
        ValidationResult::Valid(None)
    } else {
        ValidationResult::NotValid
    }
}

fn queen(mv: Move, b: &Board) -> ValidationResult {
    match (bishop(mv, b), rook(mv, b)) {
        (ValidationResult::Valid(_), _) | (_, ValidationResult::Valid(_)) => {
            ValidationResult::Valid(None)
        }
        _ => ValidationResult::NotValid,
    }
    // if bishop(mv, b) || rook(mv, b) {
    //     ValidationResult::Valid(None)
    // } else {
    //     ValidationResult::NotValid
    // }
}

fn king(mv: Move, _: &Board) -> ValidationResult {
    if mv.columns.abs() <= 1 && mv.rows.abs() <= 1 {
        ValidationResult::Valid(None)
    } else {
        ValidationResult::NotValid
    }
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
