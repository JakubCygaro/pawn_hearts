use std::ffi::CString;
use std::str::FromStr;

use raylib::color::Color;
use raylib::drawing::RaylibDrawHandle;
use raylib::ffi::{KeyboardKey, MeasureTextEx, MouseButton};
use raylib::math as rmath;
use raylib::prelude::RaylibDraw;

#[derive(Clone, Copy)]
pub struct FontWrap<'a> {
    font: &'a raylib::text::Font,
    sz: f32,
    sp: f32,
}

impl<'a> FontWrap<'a> {
    pub fn wrap(font: &'a raylib::text::Font, size: f32, spacing: f32) -> Self {
        Self {
            font,
            sz: size,
            sp: spacing,
        }
    }
}

pub fn measure_text_ex(
    font: &raylib::text::Font,
    text: &str,
    font_size: f32,
    spacing: f32,
) -> rmath::Vector2 {
    let cstr = CString::from_str(text).unwrap();
    unsafe { MeasureTextEx(**font, cstr.as_ptr(), font_size, spacing).into() }
}

pub fn button(
    hndl: &mut RaylibDrawHandle,
    pos: rmath::Vector2,
    text: &str,
    font: FontWrap,
) -> (bool, rmath::Vector2) {
    let tsz = measure_text_ex(font.font, text, font.sz, font.sp);
    let button_sz = tsz * 1.5;
    let center = pos - button_sz / 2.;

    let mouse_pos = hndl.get_mouse_position();
    let rec = rmath::Rectangle {
        x: center.x,
        y: center.y,
        width: button_sz.x,
        height: button_sz.y,
    };
    let (c, r) = if rmath::Rectangle::check_collision_point_rec(&rec, mouse_pos) {
        (
            Color::ORANGE,
            hndl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
        )
    } else {
        (Color::YELLOW, false)
    };
    hndl.draw_rectangle_v(center, button_sz, c);
    hndl.draw_text_ex(
        font.font,
        text,
        pos - (tsz / 2.0),
        font.sz,
        font.sp,
        Color::BLACK,
    );
    (r, button_sz)
}

pub fn text_input(
    hndl: &mut RaylibDrawHandle,
    pos: rmath::Vector2,
    text: &mut String,
    font: FontWrap,
) -> (bool, rmath::Vector2) {
    if let Some(c) = hndl.get_key_pressed() {
        (c == KeyboardKey::KEY_BACKSPACE).then(|| text.pop());
    }
    if let Some(c) = hndl.get_char_pressed() {
        text.push(c)
    }
    let text = if text.is_empty() {
        "<Empty>"
    } else {
        text.as_str()
    };
    let tsz = measure_text_ex(font.font, text, font.sz, font.sp);
    let area_sz = rmath::Vector2 {
        x: tsz.x + 2.0 * font.sp,
        y: tsz.y * 1.5,
    };
    let center = pos - area_sz / 2.;
    hndl.draw_rectangle_v(center, area_sz, Color::GRAY);
    hndl.draw_text_ex(
        font.font,
        text,
        pos - (tsz / 2.0),
        font.sz,
        font.sp,
        Color::BLACK,
    );
    (false, area_sz)
}
