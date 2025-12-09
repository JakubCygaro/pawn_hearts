use std::ffi::CString;
use std::str::FromStr;

use raylib::color::Color;
use raylib::drawing::RaylibDrawHandle;
use raylib::ffi::{GetFontDefault, MeasureTextEx};
use raylib::math as rmath;
use raylib::prelude::RaylibDraw;

pub fn measure_text_ex(
    font: raylib::ffi::Font,
    text: &str,
    font_size: f32,
    spacing: f32,
) -> rmath::Vector2 {
    let cstr = CString::from_str(text).unwrap();
    unsafe { MeasureTextEx(font, cstr.as_ptr(), font_size, spacing).into() }
}

pub fn measure_default_text_ex(text: &str, font_size: f32, spacing: f32) -> rmath::Vector2 {
    unsafe { measure_text_ex(GetFontDefault(), text, font_size, spacing) }
}

pub fn button(hndl: &mut RaylibDrawHandle, pos: rmath::Vector2, text: &str) -> bool {
    let tsz = measure_default_text_ex(text, 24.0, 12.0);
    let button_sz = tsz * 1.5;
    let center = pos - button_sz / 2.;
    hndl.draw_rectangle_v(center, button_sz, Color::YELLOW);
    hndl.draw_text_ex(hndl.get_font_default(), text, pos - (tsz / 2.0), 24.0, 12.0, Color::BLACK);
    false
}
