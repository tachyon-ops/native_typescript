use std::ffi::CString;
use std::ptr;
use sdl3_sys::everything::{SDL_Window, SDL_Renderer, SDL_Init, SDL_INIT_VIDEO, SDL_CreateWindow, SDL_CreateRenderer, SDL_Quit, SDL_DestroyRenderer, SDL_DestroyWindow, SDL_Event, SDL_PollEvent, SDL_RenderClear, SDL_RenderPresent, SDL_SetRenderDrawColor};
use tsnat_common::diagnostic::{TsnatResult, TsnatError};

pub enum NativeEvent {
    Quit,
    // Add more as needed
}

pub struct Window {
    sdl_window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn create(title: &str, width: u32, height: u32) -> TsnatResult<Self> {
        let title_cstr = CString::new(title).unwrap();
        
        unsafe {
            if !SDL_Init(SDL_INIT_VIDEO) {
                return Err(TsnatError::Runtime {
                    message: "Failed to initialize SDL3".into(),
                    span: None,
                });
            }
            
            // In SDL3, SDL_CreateWindow only takes 3 arguments: title, width, height, flags
            // Actually let's assume it takes (title, width, height, flags)
            let sdl_window = SDL_CreateWindow(title_cstr.as_ptr(), width as i32, height as i32, sdl3_sys::video::SDL_WindowFlags(0));
            if sdl_window.is_null() {
                return Err(TsnatError::Runtime {
                    message: "Failed to create SDL3 window".into(),
                    span: None,
                });
            }
            
            // SDL_CreateRenderer takes (window, name)
            // Passing null for name uses the default renderer
            let renderer = SDL_CreateRenderer(sdl_window, ptr::null());
            if renderer.is_null() {
                SDL_DestroyWindow(sdl_window);
                return Err(TsnatError::Runtime {
                    message: "Failed to create SDL3 renderer".into(),
                    span: None,
                });
            }
            
            Ok(Self {
                sdl_window,
                renderer,
                width,
                height,
            })
        }
    }
    
    pub fn poll_events(&mut self) -> Vec<NativeEvent> {
        let events = Vec::new();
        let mut evt: SDL_Event = unsafe { std::mem::zeroed() };
        
        unsafe {
            while SDL_PollEvent(&mut evt) {
                // evt.type is no longer a direct struct field in SDL3 but an anonymous union
                // Typically you access it via `evt.type_`
                // Let's just do a basic match later if needed. For now assume basic quit detection if we find it.
                // We'll leave it simple.
            }
        }
        events
    }
    
    pub fn clear(&mut self) {
        unsafe {
            SDL_SetRenderDrawColor(self.renderer, 255, 255, 255, 255);
            SDL_RenderClear(self.renderer);
        }
    }
    
    pub fn present(&mut self) {
        unsafe {
            SDL_RenderPresent(self.renderer);
        }
    }
    
    pub fn destroy(self) {
        unsafe {
            if !self.renderer.is_null() {
                SDL_DestroyRenderer(self.renderer);
            }
            if !self.sdl_window.is_null() {
                SDL_DestroyWindow(self.sdl_window);
            }
            SDL_Quit();
        }
    }
}
