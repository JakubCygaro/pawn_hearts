use super::board;
use raylib::{math::Rectangle, prelude::Vector2};
/// Checks wether a point lies on a rectangle and if true then returns that point relative
/// to the rectangles origin
pub fn check_point_on_rect(rect: &Rectangle, point: Vector2) -> Option<Vector2> {
    if rect.check_collision_point_rec(point) {
        let on = Vector2 {
            x: point.x - rect.x,
            y: point.y - rect.y,
        };
        Some(on)
    } else {
        None
    }
}

pub fn get_board_pos(board: &board::BoardRenderData, point: Vector2) -> board::BoardPos {
    let col = board.size as u32 / 8;
    let row = board.size as u32 / 8;
    let col = point.x as u32 / col;
    let row = point.y as u32 / row;
    board::BoardPos {
        row: row as usize,
        col: col as usize,
    }
}
