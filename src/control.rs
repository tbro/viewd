use std::path::Path;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;
use tokio::sync::mpsc::channel;

use crate::db::DbDropGuard;
use crate::sdl_window::SdlWindow;
use crate::server;

/// Spawns TPCListner task and initialized SdlWindow control loop. Database
/// is initialized and passed to both for shared state. Mpsc channel is used to
/// transmit commands from Tcp handler to Sdl Window.
pub async fn run<A: ToSocketAddrs>(addr: A, path: &Path) -> crate::Result<()> {
    let db_holder = DbDropGuard::new();
    let db = db_holder.db();

    let (win_cmd_tx, win_cmd_rx) = channel(32);
    let listener = TcpListener::bind(addr).await?;

    tokio::spawn(async move {
        server::run(listener, db_holder, win_cmd_tx, signal::ctrl_c()).await;
    });

    let mut window = SdlWindow::new("viewd", path, win_cmd_rx, db)?;

    let _ = window.init();
    window.handle_event()?;

    Ok(())
}
