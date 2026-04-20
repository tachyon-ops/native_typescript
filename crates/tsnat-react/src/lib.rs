pub mod window;
pub mod layout;
pub mod font;
pub mod render;
pub mod reconciler;
pub mod entry;

pub use layout::*;
pub use font::*;

pub struct TestRenderer;

impl TestRenderer {
    pub fn get_text(&self, _test_id: &str) -> String {
        String::new()
    }

    pub fn press(&self, _test_id: &str) {}

    pub fn find(&self, _test_id: &str) -> Option<()> {
        Some(())
    }

    pub fn get_layout(&self, _test_id: &str) -> LayoutResult {
        LayoutResult { width: 0.0, height: 0.0, x: 0.0, y: 0.0 }
    }

    pub fn get_global_number(&self, _name: &str) -> f64 {
        0.0
    }
}

pub struct LayoutResult {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

pub struct RenderContext;

impl RenderContext {
    pub fn new_headless() -> Self {
        Self
    }

    pub fn eval_and_render(&mut self, _src: &str) -> Result<TestRenderer, ()> {
        Ok(TestRenderer)
    }
}
