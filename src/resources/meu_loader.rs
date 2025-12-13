use anyhow::{anyhow, Result};
use meurglys3_lib;
use raylib::text::Font;
use raylib::texture::Texture2D;
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
                "png" => {
                    let s = f.to_str().unwrap().to_owned();
                    let t = handle.load_texture(thread, s.as_str())?;
                    textures.insert(s, Rc::from(t));
                }
                "otf" | "ttf" => {
                    let s = f.to_str().unwrap().to_owned();
                    let font = handle.load_font(thread, s.as_str())?;
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
