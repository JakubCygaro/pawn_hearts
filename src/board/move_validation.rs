use super::{BoardMove, BoardPos, ChessBoard, ChessBoardCell, ChessPiece, LongStart};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub enum SideEffect {
    Delete(BoardPos, ChessBoardCell),
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
type VRes = ValidationResult;
type SEffect = SideEffect;
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
            (Cell::Black(Piece::King(true)), king as MoveChecker),
            (Cell::White(Piece::King(true)), king as MoveChecker),
            (
                Cell::Black(Piece::King(false)),
                black_king_can_castle as MoveChecker,
            ),
            (
                Cell::White(Piece::King(false)),
                white_king_can_castle as MoveChecker,
            ),
        ])
    };
}

fn black_pawn(mv: Move, b: &Board) -> VRes {
    match mv {
        Move {
            rows: 1,
            columns: -1,
            ..
        }
        | Move {
            rows: 1,
            columns: 1,
            ..
        } => {
            if let Some(&Cell::White(_)) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::Black(Piece::Pawn(LongStart::After)),
                )]))
            } else if let Some(&Cell::White(Piece::Pawn(LongStart::RightNow))) = b.at(BoardPos {
                row: mv.from.row,
                col: mv.from.col - 1,
            }) {
                VRes::Valid(Some(vec![SEffect::Delete(
                    BoardPos {
                        row: mv.from.row,
                        col: mv.from.col - 1,
                    },
                    *b.at(BoardPos {
                        row: mv.from.row,
                        col: mv.from.col - 1,
                    })
                    .unwrap(),
                )]))
            } else if let Some(&Cell::White(Piece::Pawn(LongStart::RightNow))) = b.at(BoardPos {
                row: mv.from.row,
                col: mv.from.col + 1,
            }) {
                VRes::Valid(Some(vec![SEffect::Delete(
                    BoardPos {
                        row: mv.from.row,
                        col: mv.from.col + 1,
                    },
                    *b.at(BoardPos {
                        row: mv.from.row,
                        col: mv.from.col + 1,
                    })
                    .unwrap(),
                )]))
            } else {
                VRes::NotValid
            }
        }
        Move {
            rows: 2,
            columns: 0,
            ..
        } if mv.from.row == 1 => {
            if let Some(&Cell::Empty) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::Black(Piece::Pawn(LongStart::RightNow)),
                )]))
            } else {
                VRes::NotValid
            }
        }
        Move {
            rows: 1,
            columns: 0,
            ..
        } => {
            if let Some(&Cell::Empty) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::Black(Piece::Pawn(LongStart::After)),
                )]))
            } else {
                VRes::NotValid
            }
        }
        _ => {
            VRes::NotValid
        }
    }
}

fn white_pawn(mv: Move, b: &Board) -> VRes {
    match mv {
        Move {
            rows: -1,
            columns: -1,
            ..
        }
        | Move {
            rows: -1,
            columns: 1,
            ..
        } => {
            if let Some(&Cell::Black(_)) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::White(Piece::Pawn(LongStart::After)),
                )]))
            } else if let Some(&Cell::Black(Piece::Pawn(LongStart::RightNow))) = b.at(BoardPos {
                row: mv.from.row,
                col: mv.from.col - 1,
            }) {
                VRes::Valid(Some(vec![SEffect::Delete(
                    BoardPos {
                        row: mv.from.row,
                        col: mv.from.col - 1,
                    },
                    *b.at(BoardPos {
                        row: mv.from.row,
                        col: mv.from.col - 1,
                    })
                    .unwrap(),
                )]))
            } else if let Some(&Cell::Black(Piece::Pawn(LongStart::RightNow))) = b.at(BoardPos {
                row: mv.from.row,
                col: mv.from.col + 1,
            }) {
                VRes::Valid(Some(vec![SEffect::Delete(
                    BoardPos {
                        row: mv.from.row,
                        col: mv.from.col + 1,
                    },
                    *b.at(BoardPos {
                        row: mv.from.row,
                        col: mv.from.col + 1,
                    })
                    .unwrap(),
                )]))
            } else {
                VRes::NotValid
            }
        }
        Move {
            rows: -2,
            columns: 0,
            ..
        } if mv.from.row == 6 => {
            if let Some(&Cell::Empty) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::White(Piece::Pawn(LongStart::RightNow)),
                )]))
            } else {
                VRes::NotValid
            }
        }
        Move {
            rows: -1,
            columns: 0,
            ..
        } => {
            if let Some(&Cell::Empty) = b.at(mv.to) {
                VRes::Valid(Some(vec![SEffect::SetAt(
                    mv.to,
                    Cell::White(Piece::Pawn(LongStart::After)),
                )]))
            } else {
                VRes::NotValid
            }
        }
        _ => VRes::NotValid,
    }
}

fn bishop(mv: Move, b: &Board) -> VRes {
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
                return VRes::NotValid;
            };
        }
        VRes::Valid(None)
    } else {
        VRes::NotValid
    }
}

fn rook(mv: Move, b: &Board) -> VRes {
    if mv.columns == 0 {
        for r in bisex_range(0, mv.rows).skip(1) {
            let to_check = BoardPos {
                row: (mv.from.row as isize + r) as usize,
                col: mv.from.col,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        VRes::Valid(None)
    } else if mv.rows == 0 {
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        VRes::Valid(None)
    } else {
        VRes::NotValid
    }
}

fn knight(mv: Move, _: &Board) -> VRes {
    if (mv.columns.abs() == 2 && mv.rows.abs() == 1)
        || (mv.columns.abs() == 1 && mv.rows.abs() == 2)
    {
        VRes::Valid(None)
    } else {
        VRes::NotValid
    }
}

fn queen(mv: Move, b: &Board) -> VRes {
    match (bishop(mv, b), rook(mv, b)) {
        (VRes::Valid(_), _) | (_, VRes::Valid(_)) => VRes::Valid(None),
        _ => VRes::NotValid,
    }
}
fn black_king_can_castle(mv: Move, b: &Board) -> VRes {
    //close castling
    if mv.from.row == 0 && mv.from.col == 4 && mv.to.row == 0 && mv.to.col == 6 {
        let Some(Cell::Black(Piece::Rook)) = b.at(BoardPos { row: 0, col: 7 }) else {
            return VRes::NotValid;
        };
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        return VRes::Valid(Some(vec![
            SEffect::Move(Move {
                from: BoardPos { row: 0, col: 7 },
                to: BoardPos { row: 0, col: 5 },
                rows: 0,
                columns: -2,
            }),
            SEffect::SetAt(mv.to, Cell::Black(Piece::King(true))),
        ]));
    } else if mv.from.row == 0 && mv.from.col == 4 && mv.to.row == 0 && mv.to.col == 1 {
        //long castling
        let Some(Cell::Black(Piece::Rook)) = b.at(BoardPos { row: 0, col: 0 }) else {
            return VRes::NotValid;
        };
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        return VRes::Valid(Some(vec![
            SEffect::Move(Move {
                from: BoardPos { row: 0, col: 0 },
                to: BoardPos { row: 0, col: 2 },
                rows: 0,
                columns: 2,
            }),
            SEffect::SetAt(mv.to, Cell::Black(Piece::King(true))),
        ]));
    }
    if let VRes::Valid(None) = king(mv, b) {
        VRes::Valid(Some(vec![SEffect::SetAt(
            mv.to,
            Cell::Black(Piece::King(true)),
        )]))
    } else {
        VRes::NotValid
    }
}
fn white_king_can_castle(mv: Move, b: &Board) -> VRes {
    //close castling
    if mv.from.row == 7 && mv.from.col == 4 && mv.to.row == 7 && mv.to.col == 6 {
        let Some(Cell::White(Piece::Rook)) = b.at(BoardPos { row: 7, col: 7 }) else {
            return VRes::NotValid;
        };
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        return VRes::Valid(Some(vec![
            SEffect::Move(Move {
                from: BoardPos { row: 7, col: 7 },
                to: BoardPos { row: 7, col: 5 },
                rows: 0,
                columns: -2,
            }),
            SEffect::SetAt(mv.to, Cell::White(Piece::King(true))),
        ]));
    } else if mv.from.row == 7 && mv.from.col == 4 && mv.to.row == 7 && mv.to.col == 1 {
        //long castling
        let Some(Cell::White(Piece::Rook)) = b.at(BoardPos { row: 7, col: 0 }) else {
            return VRes::NotValid;
        };
        for c in bisex_range(0, mv.columns).skip(1) {
            let to_check = BoardPos {
                row: mv.from.row,
                col: (mv.from.col as isize + c) as usize,
            };
            let Some(Cell::Empty) = b.at(to_check) else {
                return VRes::NotValid;
            };
        }
        return VRes::Valid(Some(vec![
            SEffect::Move(Move {
                from: BoardPos { row: 7, col: 0 },
                to: BoardPos { row: 7, col: 2 },
                rows: 0,
                columns: 2,
            }),
            SEffect::SetAt(mv.to, Cell::White(Piece::King(true))),
        ]));
    }
    if let VRes::Valid(None) = king(mv, b) {
        VRes::Valid(Some(vec![SEffect::SetAt(
            mv.to,
            Cell::White(Piece::King(true)),
        )]))
    } else {
        VRes::NotValid
    }
}

fn king(mv: Move, _: &Board) -> VRes {
    if mv.columns.abs() <= 1 && mv.rows.abs() <= 1 {
        VRes::Valid(None)
    } else {
        VRes::NotValid
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
