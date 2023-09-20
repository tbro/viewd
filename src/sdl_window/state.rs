use anyhow::{anyhow, Result};
use sdl2::video::FullscreenType;
use std::ffi::{OsStr, OsString};

/// Struct to hold Window State
pub struct WindowState {
    /// Degrees to rotate the image
    rotation: f64,
    /// Fullscreen mode
    fullscreen: FullscreenType,
    /// Title shown at the top of the window
    title: OsString,
}

impl WindowState {
    pub fn new(title: &str) -> Self {
        let title = OsString::from(title);
        let fullscreen = FullscreenType::Off;
        let rotation: f64 = 0.0;
        Self {
            title,
            fullscreen,
            rotation,
        }
    }
    pub fn fullscreen(&self) -> FullscreenType {
        self.fullscreen
    }
    pub fn rotation(&self) -> f64 {
        self.rotation
    }
    pub fn title(&self) -> Result<&str> {
        let title = self
            .title
            .to_str()
            .ok_or(anyhow!("could not convert OsString to str"))?;
        Ok(title)
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
