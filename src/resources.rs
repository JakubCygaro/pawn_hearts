use anyhow::{anyhow, Result};
use raylib::text::Font;
use raylib::texture::Texture2D;
use std::{collections::HashMap, path::PathBuf, rc::Rc};
pub trait ResourceLoader {
    fn get_texture(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Texture2D>>;
    fn get_texture_no_load(&self, path: &str) -> Option<Rc<Texture2D>>;
    fn get_font(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    )-> Result<Rc<Font>>;
    fn get_font_no_load(
        &self,
        path: &str,
    )-> Option<Rc<Font>>;
}

pub struct DirectoryResourceLoader {
    root_dir: PathBuf,
    textures: HashMap<String, Rc<Texture2D>>,
    fonts: HashMap<String, Rc<Font>>,
}

impl DirectoryResourceLoader {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }
    fn load_texture(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Texture2D>> {
        let mut file_path = self.root_dir.clone();
        file_path.push(path);

        let texture = handle
            .load_texture(thread, file_path.to_str().unwrap())
            .map_err(|e| anyhow!(e))?;
        let texture = Rc::new(texture);
        self.textures.insert(path.to_string(), texture.clone());
        Ok(texture)
    }
    fn load_font(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Font>> {
        let mut file_path = self.root_dir.clone();
        file_path.push(path);

        let font = handle
            .load_font(thread, file_path.to_str().unwrap())
            .map_err(|e| anyhow!(e))?;
        let font = Rc::new(font);
        self.fonts.insert(path.to_string(), font.clone());
        Ok(font)
    }
    pub fn load_all_root(
        &mut self,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<()> {
        let files = get_files_recurse(&self.root_dir)?;
        for f in files.iter() {
            let ext = f.extension().unwrap().to_str().unwrap();
            match ext {
                "png" => self.load_texture(f.to_str().unwrap(), handle, thread).map(|_|())?,
                "otf" | "ttf" => self.load_font(f.to_str().unwrap(), handle, thread).map(|_|())?,
                _ => ()
            }
        }
        Ok(())
    }
}

fn get_files_recurse(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut ret = vec![];
    use std::fs;

    for file in fs::read_dir(path)? {
        let file = file?;
        let f_path = file.path();
        if f_path.is_dir() {
            let mut inner = get_files_recurse(&f_path)?;
            ret.append(&mut inner);
        } else {
            let f_path = f_path.strip_prefix(path).unwrap().to_owned();
            ret.push(f_path);
        }
    }
    Ok(ret)
}

impl ResourceLoader for DirectoryResourceLoader {
    fn get_texture(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    ) -> Result<Rc<Texture2D>> {
        if let Some(t) = self.textures.get(path) {
            Ok(t.to_owned())
        } else {
            let loaded = self.load_texture(path, handle, thread)?;
            Ok(loaded)
        }
    }
    fn get_texture_no_load(&self, path: &str) -> Option<Rc<Texture2D>> {
        self.textures.get(path).cloned()
    }
    fn get_font(
        &mut self,
        path: &str,
        handle: &mut raylib::RaylibHandle,
        thread: &mut raylib::RaylibThread,
    )-> Result<Rc<Font>>{
        if let Some(t) = self.fonts.get(path) {
            Ok(t.to_owned())
        } else {
            let loaded = self.load_font(path, handle, thread)?;
            Ok(loaded)
        }
    }
    fn get_font_no_load(
        &self,
        path: &str,
    )-> Option<Rc<Font>>{
        self.fonts.get(path).cloned()
    }
}
