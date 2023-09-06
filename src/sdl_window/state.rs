use sdl2::video::FullscreenType;
use std::ffi::{OsString, OsStr};

/// Struct to hold current state of
pub struct WindowState {
    /// Degrees to rotate the image
    rotation: f64,
    /// Fullscreen mode
    fullscreen: FullscreenType,
    /// Pageant mode advances to cursor automatically
    pageant_mode: bool,
    /// Wheather or not sufficient time has passed to advance the cursor
    pageant_ready: bool,
    /// Title shown at the top of the window
    title: OsString,
}

impl WindowState {
    pub fn new(title: &str) -> Self {
	let title = OsString::from(title);
	let fullscreen = FullscreenType::Off;
	let rotation: f64 = 0.0;
	let pageant_mode = false;
	let pageant_ready = false;
	Self { title, fullscreen, rotation, pageant_mode, pageant_ready }
    }
    pub fn fullscreen(&self) -> FullscreenType {
	self.fullscreen
    }
    pub fn rotation(&self) -> f64 {
	self.rotation
    }
    pub fn title(&self) -> &str {
	self.title.to_str().unwrap()
    }
    fn _pageant(&self) -> bool {
	self.pageant_mode
    }
    pub fn toggle_fullscreen(&mut self) -> FullscreenType {
	match self.fullscreen {
	    FullscreenType::Off => self.fullscreen = FullscreenType::Desktop,
	    FullscreenType::True => self.fullscreen = FullscreenType::Off,
	    FullscreenType::Desktop => self.fullscreen = FullscreenType::Off,
	};
	self.fullscreen
    }
    fn _toggle_pageant(&mut self) {
	self.pageant_mode = !self.pageant_mode;
    }
    pub fn rotate(&mut self, f: f64) -> f64 {
	self.rotation += f;
	self.rotation
    }
    pub fn set_title(&mut self, s: &OsStr) {
	self.title = s.into();
    }
}
