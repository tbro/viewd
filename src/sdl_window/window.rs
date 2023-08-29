use anyhow::{anyhow, Result};
use sdl2::EventPump;

use sdl2::image::LoadTexture;
use sdl2::{render::WindowCanvas, video::FullscreenType};
use std::ffi::OsString;
use std::path::Path;
use tracing::debug;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;

use crate::db::Db;
use crate::window::WindowCommand;

use super::navigator::Navigator;

/// Wrapper for all Window related logic.
pub struct SdlWindow {
    /// Degrees to rotate the image
    rotation: f64,
    /// Fullscreen mode
    fullscreen: FullscreenType,
    /// Pageant mode advances to cursor automatically
    pageant_mode: bool,
    /// Wheather or not sufficient time has passed to advance the cursor
    pageant_ready: bool,
    /// Sdl Window Canvas to draw images on
    canvas: WindowCanvas,
    /// Title shown at the top of the window
    window_title: OsString,
    /// Listener to receive commands from the Window
    event_pump: EventPump,
    /// Cursor of file paths to display in the Window
    cursor: Navigator,
    /// Receiver for Commands received over TCP
    rx: Receiver<WindowCommand>,
    /// Used to cleanly exit Window control loop
    shutdown: Arc<Mutex<bool>>,
    /// Db instance to mutate when the display image is updated
    /// (cursor is advanced).
    db: Db,
}

impl SdlWindow {
    pub(crate) fn new(
        title: &str,
        path: &Path,
        rx: Receiver<WindowCommand>,
        db: Db,
    ) -> Result<Self> {
        let title = OsString::from(title);
        let cursor = Navigator::new(path)?;
        let sdl_context = sdl2::init().map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let fullscreen = FullscreenType::Off;
        let rotation: f64 = 0.0;
        let pageant_mode = false;
        let pageant_ready = false;
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let window = video_subsystem
            .window("viewd", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .software()
            .target_texture()
            .build()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;

        let shutdown = Arc::new(Mutex::new(false));
        let s = Self {
            fullscreen,
            rotation,
            pageant_mode,
            canvas,
            window_title: title,
            pageant_ready,
            event_pump,
            cursor,
            rx,
            shutdown,
            db,
        };

        Ok(s)
    }
    fn next(&mut self) -> Result<()> {
        self.cursor.next();
        self.update()?;
        Ok(())
    }
    fn prev(&mut self) -> Result<()> {
        self.cursor.prev();
        self.update()?;
        Ok(())
    }
    /// wraps update methods
    fn update(&mut self) -> Result<()> {
        self.update_title();
        self.update_canvas()?;
        self.update_window()?;
        Ok(())
    }
    pub(crate) fn init(&mut self) -> Result<()> {
        self.cursor.next().unwrap();
        self.update()?;
        Ok(())
    }
    /// Update window_title on Self and `display_path` in db.
    fn update_title(&mut self) {
        if let Some(name) = self.cursor.image.file_name() {
            self.db.set("display_path".to_string(), name.into());
            self.window_title = name.to_owned();
        }
    }
    fn update_canvas(&mut self) -> Result<()> {
        self.canvas.clear();
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .load_texture(self.cursor.image.clone())
            .map_err(|e| anyhow!("Update Canvas Error: {}", e))?;
        self.canvas
            .copy_ex(
                &texture,
                None,
                None,
                self.rotation * -90_f64,
                None,
                false,
                false,
            )
            .map_err(|e| anyhow!("Update Canvas Error: {}", e))?;
        self.canvas.present();
        Ok(())
    }
    fn update_window(&mut self) -> Result<()> {
        let window = self.canvas.window_mut();
        window.set_fullscreen(self.fullscreen).unwrap();
        window
            .set_title(self.window_title.to_str().unwrap())
            .map_err(|e| anyhow!("Update Window Error: {}", e))?;
        Ok(())
    }
    fn fullscreen_toggle(&mut self) -> Result<()> {
        match self.fullscreen {
            FullscreenType::Off => self.fullscreen = FullscreenType::Desktop,
            FullscreenType::True => self.fullscreen = FullscreenType::Off,
            FullscreenType::Desktop => self.fullscreen = FullscreenType::Off,
        };

        let window = self.canvas.window_mut();
        window
            .set_fullscreen(self.fullscreen)
            .map_err(|e| anyhow!("FullScreen Toggle Error: {}", e))?;
        self.update_canvas()?;
        Ok(())
    }
    fn _pageant_toggle(&mut self) {
        self.pageant_mode = !self.pageant_mode;
    }
    fn rotate(&mut self, f: f64) -> Result<()> {
        self.rotation += f;
        self.update_canvas()?;
        Ok(())
    }
    fn _try_load(&mut self, image: &Path) -> Option<()> {
        let texture_creator = self.canvas.texture_creator();
        texture_creator.load_texture(image).ok().map(|_| ())
    }
    pub(crate) fn handle_event(&mut self) -> Result<()> {
        loop {
            if *self.shutdown.lock().unwrap() {
                break Ok(());
            }
            while let Ok(command) = self.rx.try_recv() {
                debug!(?command);

                match command {
                    WindowCommand::Quit => *self.shutdown.lock().unwrap() = true,
                    WindowCommand::Next => {
                        self.next()?;
                    }
                    WindowCommand::Prev => {
                        self.prev()?;
                    }
                    WindowCommand::Fullscreen => {
                        self.fullscreen_toggle()?;
                    }
                    WindowCommand::Rotate => {
                        self.rotate(1.0)?;
                    }
                }
            }

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                        ..
                    } => *self.shutdown.lock().unwrap() = true,
                    _ => {}
                };
            }
        }
    }
}
