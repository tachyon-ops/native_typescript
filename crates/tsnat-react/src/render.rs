// tsnat-react/src/render.rs
use crate::window::{Window, NativeEvent};
use crate::layout::LayoutTree;
use crate::font::FontRenderer;
use std::f32;
use std::collections::HashMap;

#[derive(Debug)]
pub enum IntrinsicTag {
    Div,
    Span,
}

pub struct Widget {
    pub id: u32,
    pub tag: IntrinsicTag,
    pub text_node: Option<String>,
    pub children: Vec<u32>,
}

pub struct Application {
    pub window: Window,
    pub layout: LayoutTree,
    pub font_renderer: FontRenderer,
    pub widgets: HashMap<u32, Widget>,
    pub next_id: u32,
    pub root_widget: Option<u32>,
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
            widgets: HashMap::new(),
            next_id: 1,
            root_widget: None,
        })
    }

    pub fn create_widget(&mut self, tag: IntrinsicTag, text: Option<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        self.widgets.insert(id, Widget {
            id,
            tag,
            text_node: text,
            children: Vec::new(),
        });
        
        self.layout.insert_node(id);
        id
    }

    pub fn append_child(&mut self, parent_id: u32, child_id: u32) {
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            parent.children.push(child_id);
            let index = parent.children.len() as u32 - 1;
            self.layout.insert_child(parent_id, child_id, index);
        }
    }

    pub fn remove_child(&mut self, parent_id: u32, child_id: u32) {
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            parent.children.retain(|&c| c != child_id);
            self.layout.remove_child(parent_id, child_id);
        }
    }

    pub fn insert_before(&mut self, parent_id: u32, child_id: u32, before_child_id: u32) {
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            if let Some(index) = parent.children.iter().position(|&c| c == before_child_id) {
                parent.children.insert(index, child_id);
                self.layout.insert_child(parent_id, child_id, index as u32);
            } else {
                parent.children.push(child_id);
                let idx = parent.children.len() as u32 - 1;
                self.layout.insert_child(parent_id, child_id, idx);
            }
        }
    }
    
    pub fn get_root(&self) -> Option<u32> {
        self.root_widget
    }

    pub fn set_root(&mut self, id: u32) {
        self.layout.set_root(id);
        self.root_widget = Some(id);
        self.window.show();
    }

    fn get_topological_order(&self) -> Vec<u32> {
        let mut order = Vec::new();
        if let Some(root) = self.root_widget {
            self.visit_widget(root, &mut order);
        }
        order
    }

    fn visit_widget(&self, id: u32, order: &mut Vec<u32>) {
        order.push(id);
        if let Some(w) = self.widgets.get(&id) {
            for child in &w.children {
                self.visit_widget(*child, order);
            }
        }
    }

    pub fn tick(&mut self) -> Option<Vec<u32>> {
        let mut clicked_widgets = Vec::new();
        let topo_ids = self.get_topological_order();

        // Poll input events
        let events = self.window.poll_events();
        for ev in events {
            match ev {
                NativeEvent::Quit => return None,
                NativeEvent::MouseClick { x: cx, y: cy } => {
                    let mut reverse_topo = topo_ids.clone();
                    reverse_topo.reverse(); // check children first
                    
                    for id in reverse_topo {
                        if let Some((mut x, mut y, mut w, mut h)) = self.layout.get_layout(id) {
                            if x.is_nan() { x = 0.0; }
                            if y.is_nan() { y = 0.0; }
                            if w.is_nan() { w = 0.0; }
                            if h.is_nan() { h = 0.0; }
                            
                            // Let's assume standard width and height exist. If text context, bounds might be 0, but Yoga text measure would set w/h once implemented.
                            // Temporary: Expand click area for text blocks with 0 width
                            let click_w = if w == 0.0 { 100.0 } else { w };
                            let click_h = if h == 0.0 { 50.0  } else { h };
                            
                            if cx >= x && cx <= x + click_w && cy >= y && cy <= y + click_h {
                                clicked_widgets.push(id);
                                break; // Only click the topmost widget
                            }
                        }
                    }
                }
            }
        }
        
        // Calculate Yoga bounds
        self.layout.calculate_layout(self.window.width as f32, self.window.height as f32);
        
        self.window.clear();
        
        // Iterate UI nodes using topological order
        for id in topo_ids {
            let widget = self.widgets.get(&id).unwrap();
            if let Some((mut x, mut y, mut w, mut h)) = self.layout.get_layout(id) {
                if x.is_nan() { x = 0.0; }
                if y.is_nan() { y = 0.0; }
                if w.is_nan() { w = 0.0; }
                if h.is_nan() { h = 0.0; }
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
                                // Default rendering to padded coordinate if Yoga yields 0 width bounds
                                let mut cursor_x = if w <= 0.0 { 100.0 } else { x };
                                let cursor_y = if h <= 0.0 { 100.0 } else { y + h / 2.0 };

                                unsafe {
                                    // Set blend mode to support standard font antialiasing (alpha blending)
                                    sdl3_sys::everything::SDL_SetRenderDrawBlendMode(self.window.renderer, sdl3_sys::everything::SDL_BLENDMODE_BLEND);

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
                                                            let rect = sdl3_sys::everything::SDL_FRect { x: px, y: py, w: 1.0, h: 1.0 };
                                                            sdl3_sys::everything::SDL_SetRenderDrawColor(self.window.renderer, 0, 0, 0, alpha);
                                                            sdl3_sys::everything::SDL_RenderFillRect(self.window.renderer, &rect);
                                                        }
                                                    }
                                                }
                                            }
                                            cursor_x += (glyph.advance >> 6) as f32;
                                        }
                                    }
                                    
                                    // Reset blend mode just in case
                                    sdl3_sys::everything::SDL_SetRenderDrawBlendMode(self.window.renderer, sdl3_sys::everything::SDL_BLENDMODE_NONE);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        self.window.present();
        
        Some(clicked_widgets)
    }
}
