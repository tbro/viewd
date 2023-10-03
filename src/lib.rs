pub mod serve;

pub use serve::Listener;

pub mod clients;
pub use clients::Client;

pub mod cmd;
pub use cmd::Command;

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::Frame;

mod db;
use db::Db;

mod shutdown;
use shutdown::Shutdown;

mod parse;
use parse::Parse;

pub mod control;
pub mod server;

pub mod window;
use window::WindowCommand;

pub mod sdl_window;

pub mod tui;
pub use tui::Tui;

/// Used if no port is specified.
pub const DEFAULT_PORT: u16 = 6379;

/// A specialized `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
/// This is defined as a convenience.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
