use sdl2::video::FullscreenType;
use std::ffi::{OsString, OsStr};

use super::pageant::PageantMode;

/// Struct to hold current state of
pub struct WindowState {
    /// Degrees to rotate the image
    rotation: f64,
    /// Fullscreen mode
    fullscreen: FullscreenType,
    /// Title shown at the top of the window
    title: OsString,
    /// Logic to automatically advance cursor
    pageant: PageantMode,
}

impl WindowState {
    pub fn new(title: &str) -> Self {
	let title = OsString::from(title);
	let fullscreen = FullscreenType::Off;
	let rotation: f64 = 0.0;
        let pageant = PageantMode::new();
	let pageant_mode = false;
	let pageant_ready = false;
	Self { title, fullscreen, rotation, pageant }
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
    pub fn toggle_fullscreen(&mut self) -> FullscreenType {
	match self.fullscreen {
	    FullscreenType::Off => self.fullscreen = FullscreenType::Desktop,
	    FullscreenType::True => self.fullscreen = FullscreenType::Off,
	    FullscreenType::Desktop => self.fullscreen = FullscreenType::Off,
	};
	self.fullscreen
    }
    pub fn rotate(&mut self, f: f64) -> f64 {
	self.rotation += f;
	self.rotation
    }
    pub fn set_title(&mut self, s: &OsStr) {
	self.title = s.into();
    }
}
