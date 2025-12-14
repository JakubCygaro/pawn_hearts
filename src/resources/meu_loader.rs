use anyhow::{anyhow, Result};
use meurglys3_lib;
use raylib::ffi;
use raylib::text::Font;
use raylib::texture::{Image, Texture2D};
use std::ffi::CString;
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf, rc::Rc};

pub struct MeurglisResourceLoader {
    textures: HashMap<String, Rc<Texture2D>>,
    fonts: HashMap<String, Rc<Font>>,
}

impl MeurglisResourceLoader {
    pub fn load_package(
        path: PathBuf,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<Self> {
        let package = meurglys3_lib::load_package(path)?;
        let files = package
            .get_names()
            .keys()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let mut textures = HashMap::new();
        let mut fonts = HashMap::new();

        for f in files.iter() {
            let ext = f.extension().unwrap().to_str().unwrap();
            match ext {
                ty @ "png" => {
                    let s = f.to_str().unwrap().to_owned();
                    let d = package.get_data_ref(&s).unwrap();
                    let t = unsafe {
                        let ty = CString::from_str(ty).unwrap();
                        let i = ffi::LoadImageFromMemory(ty.as_ptr(), d.as_ptr(), d.len() as i32);
                        let t = ffi::LoadTextureFromImage(i);
                        Texture2D::from_raw(t)
                    };
                    textures.insert(s, Rc::from(t));
                }
                ty @ ("otf" | "ttf") => {
                    let s = f.to_str().unwrap().to_owned();
                    println!("{s}");
                    let d = package.get_data_ref(&s).unwrap();
                    let font = unsafe {
                        let ty = CString::from_str(ty).unwrap();
                        let f = ffi::LoadFontFromMemory(
                            ty.as_ptr(),
                            d.as_ptr(),
                            d.len() as i32,
                            48,
                            std::ptr::null_mut(),
                            0,
                        );
                        Font::from_raw(f)
                    };
                    fonts.insert(s, Rc::from(font));
                }
                _ => (),
            }
        }
        Ok(Self { textures, fonts })
    }
}
impl super::ResourceLoader for MeurglisResourceLoader {
    fn get_font_no_load(&self, path: &str) -> Option<Rc<Font>> {
        self.fonts.get(path).cloned()
    }
    fn get_texture_no_load(&self, path: &str) -> Option<Rc<Texture2D>> {
        self.textures.get(path).cloned()
    }
    fn get_font(
        &mut self,
        path: &str,
        _handle: &mut raylib::RaylibHandle,
        _thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Font>> {
        self.get_font_no_load(path).ok_or(anyhow!("No font"))
    }
    fn get_texture(
        &mut self,
        path: &str,
        _handle: &mut raylib::RaylibHandle,
        _thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Texture2D>> {
        self.get_texture_no_load(path).ok_or(anyhow!("No texture"))
    }
}
