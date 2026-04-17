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
        let font_renderer = FontRenderer::new().map_err(|e| format!("{:?}", e))?;
        
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
        
        // Iterate UI nodes sequentially based on keys. (In correct Topological traversal later)
        if let Some((_, _, _, _)) = self.layout.get_layout(1) {
            // Because our naive shim loop doesn't have deep traversal yet, 
            // for the Hello World, we'll draw a literal background rectangle if the root exists!
            unsafe {
                // Background color for div
                let rect = SDL_FRect {
                    x: 0.0,
                    y: 0.0,
                    w: self.window.width as f32,
                    h: self.window.height as f32,
                };
                SDL_SetRenderDrawColor(self.window.renderer, 240, 240, 240, 255);
                SDL_RenderFillRect(self.window.renderer, &rect);
            }
        }
        
        self.window.present();
        
        true
    }
}
