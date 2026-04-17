// tsnat-react/src/font.rs
use freetype::Library as FtLibrary;
use freetype::Face;
use std::collections::HashMap;
use tsnat_common::diagnostic::{TsnatResult, TsnatError};

pub struct Glyph {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: i64,
    pub bitmap_data: Vec<u8>,
}

pub struct GlyphAtlas {
    pub glyphs: HashMap<char, Glyph>,
}

pub struct FontRenderer {
    ft_lib: FtLibrary,
    face: Option<Face>,
}

impl FontRenderer {
    pub fn new() -> TsnatResult<Self> {
        let ft_lib = FtLibrary::init().map_err(|_| TsnatError::Runtime {
            message: "Failed to initialize FreeType".into(),
            span: None,
        })?;
        
        Ok(Self {
            ft_lib,
            face: None,
        })
    }

    pub fn load_font(&mut self, path: &str) -> TsnatResult<()> {
        let face = self.ft_lib.new_face(path, 0).map_err(|_| TsnatError::Runtime {
            message: format!("Failed to load font at {}", path),
            span: None,
        })?;
        
        // Define default pixel size
        face.set_pixel_sizes(0, 48).unwrap();
        self.face = Some(face);
        Ok(())
    }

    pub fn rasterize_text(&self, text: &str) -> TsnatResult<GlyphAtlas> {
        let face = self.face.as_ref().ok_or_else(|| TsnatError::Runtime {
            message: "No font loaded".into(),
            span: None,
        })?;

        let mut atlas = GlyphAtlas {
            glyphs: HashMap::new(),
        };

        for ch in text.chars() {
            if atlas.glyphs.contains_key(&ch) {
                continue;
            }

            face.load_char(ch as usize, freetype::face::LoadFlag::RENDER).map_err(|_| TsnatError::Runtime {
                message: format!("Failed to render character {}", ch),
                span: None,
            })?;

            let glyph = face.glyph();
            let bitmap = glyph.bitmap();

            let width = bitmap.width() as u32;
            let height = bitmap.rows() as u32;
            let buffer = bitmap.buffer().to_vec();

            atlas.glyphs.insert(ch, Glyph {
                width,
                height,
                bearing_x: glyph.bitmap_left(),
                bearing_y: glyph.bitmap_top(),
                advance: glyph.advance().x as i64,
                bitmap_data: buffer,
            });
        }

        Ok(atlas)
    }
}
