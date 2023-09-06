use anyhow::{anyhow, Result};
use sdl2::EventPump;

use sdl2::image::LoadTexture;
use sdl2::render::WindowCanvas;
use std::path::Path;
use tracing::debug;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;

use crate::db::Db;
use crate::window::WindowCommand;

use super::WindowState;
use super::navigator::Navigator;

/// Wrapper for all Window related logic.
pub struct SdlWindow {
    /// Sdl Window Canvas to draw images on
    canvas: WindowCanvas,
    /// Listener to receive commands from the Window
    event_pump: EventPump,
    /// Cursor of file paths to display in the Window
    cursor: Navigator,
    /// Receiver for Commands received over TCP
    rx: Receiver<WindowCommand>,
    /// Used to cleanly exit Window control loop
    shutdown: Arc<Mutex<bool>>,
    /// Window state
    state: WindowState,
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
	let state = WindowState::new(title);
        let cursor = Navigator::new(path)?;
        let sdl_context = sdl2::init().map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
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
            canvas,
            event_pump,
            cursor,
            rx,
            shutdown,
	    state,
	    db
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
            self.state.set_title(name);
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
                self.state.rotation() * -90_f64,
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
        window.set_fullscreen(self.state.fullscreen()).unwrap();
        window
            .set_title(self.state.title())
            .map_err(|e| anyhow!("Update Window Error: {}", e))?;
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
			self.update_title();
                    }
                    WindowCommand::Prev => {
                        self.prev()?;
			self.update_title();
                    }
                    WindowCommand::Fullscreen => {
                        self.state.toggle_fullscreen();
			self.update_window()?;
                    }
                    WindowCommand::Rotate => {
                        self.state.rotate(1.0);
                    }
                }
		self.update_canvas()?;
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
