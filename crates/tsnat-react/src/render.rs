// tsnat-react/src/render.rs
use crate::window::{Window, NativeEvent};
use crate::layout::LayoutTree;
use crate::font::{FontRenderer, GlyphAtlas};
use sdl3_sys::everything::{SDL_FRect, SDL_SetRenderDrawColor, SDL_RenderFillRect};

#[derive(Debug)]
pub enum IntrinsicTag {
    Div,
    Span,
}

pub struct Widget {
    pub id: u32,
    pub tag: IntrinsicTag,
    pub text_node: Option<String>,
}

pub struct Application {
    pub window: Window,
    pub layout: LayoutTree,
    pub font_renderer: FontRenderer,
    pub widgets: std::collections::HashMap<u32, Widget>,
    next_id: u32,
}

impl Application {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, String> {
        let window = Window::create(title, width, height).map_err(|e| format!("{:?}", e))?;
        let layout = LayoutTree::new();
        let mut font_renderer = FontRenderer::new().map_err(|e| format!("{:?}", e))?;
        if let Err(e) = font_renderer.load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf") {
            println!("Font load failed: {:?}", e);
        }
        
        Ok(Self {
            window,
            layout,
            font_renderer,
            widgets: std::collections::HashMap::new(),
            next_id: 1,
        })
    }

    pub fn create_widget(&mut self, tag: IntrinsicTag, text: Option<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        self.widgets.insert(id, Widget {
            id,
            tag,
            text_node: text,
        });
        
        self.layout.insert_node(id);
        id
    }

    pub fn append_child(&mut self, parent_id: u32, child_id: u32) {
        // Find how many children the parent already has in its yoga logic (not deeply implemented yet, defaulting to append at end)
        // For simplicity right now, place at index 0.
        self.layout.insert_child(parent_id, child_id, 0);
    }
    
    pub fn set_root(&mut self, id: u32) {
        self.layout.set_root(id);
    }

    pub fn tick(&mut self) -> bool {
        // Poll input events
        let events = self.window.poll_events();
        for ev in events {
            match ev {
                NativeEvent::Quit => return false,
            }
        }
        
        // Calculate Yoga bounds
        self.layout.calculate_layout(self.window.width as f32, self.window.height as f32);
        
        self.window.clear();
        
        // Iterate UI nodes sorted by ID to ensure children (higher IDs) draw over parents
        let mut sorted_ids: Vec<u32> = self.widgets.keys().copied().collect();
        sorted_ids.sort();
        for id in sorted_ids {
            let widget = self.widgets.get(&id).unwrap();
            if let Some((x, y, w, h)) = self.layout.get_layout(id) {
                match widget.tag {
                    IntrinsicTag::Div => {
                        unsafe {
                            let rect = sdl3_sys::everything::SDL_FRect { x, y, w, h };
                            sdl3_sys::everything::SDL_SetRenderDrawColor(self.window.renderer, 240, 240, 240, 255);
                            sdl3_sys::everything::SDL_RenderFillRect(self.window.renderer, &rect);
                        }
                    }
                    IntrinsicTag::Span => {
                        if let Some(text) = &widget.text_node {
                            if let Ok(atlas) = self.font_renderer.rasterize_text(text) {
                                let mut cursor_x = x;
                                // Freetype offsets character bearings up into negative coordinate spaces relative to the baseline.
                                // We ensure it drops into visual padding explicitly.
                                let cursor_y = y + h / 2.0 + 64.0;

                                unsafe {
                                    for ch in text.chars() {
                                        if let Some(glyph) = atlas.glyphs.get(&ch) {
                                            for row in 0..glyph.height {
                                                for col in 0..glyph.width {
                                                    let pixel_idx = (row * glyph.width + col) as usize;
                                                    if pixel_idx < glyph.bitmap_data.len() {
                                                        let alpha = glyph.bitmap_data[pixel_idx];
                                                        if alpha > 0 {
                                                            let px = cursor_x + col as f32 + glyph.bearing_x as f32;
                                                            let py = cursor_y + row as f32 - glyph.bearing_y as f32;
                                                            let mut rect = sdl3_sys::everything::SDL_FRect { x: px, y: py, w: 1.0, h: 1.0 };
                                                            sdl3_sys::everything::SDL_SetRenderDrawColor(self.window.renderer, 0, 0, 0, alpha);
                                                            sdl3_sys::everything::SDL_SetRenderDrawBlendMode(self.window.renderer, sdl3_sys::everything::SDL_BLENDMODE_BLEND);
                                                            sdl3_sys::everything::SDL_RenderFillRect(self.window.renderer, &mut rect);
                                                        }
                                                    }
                                                }
                                            }
                                            cursor_x += (glyph.advance >> 6) as f32;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        self.window.present();
        
        true
    }
}
