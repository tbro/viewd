use std::path::Path;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;
use tokio::sync::mpsc::channel;

use crate::db::DbDropGuard;
use crate::sdl_window::SdlWindow;
use crate::serve::{get_acceptor, Config};
use crate::server;

/// Spawns TPCListener task and initialized SdlWindow control loop. Database
/// is initialized and passed to both for shared state. Mpsc channel is used to
/// transmit commands from Tcp handler to Sdl Window.
pub async fn run<A: ToSocketAddrs>(
    addr: A,
    path: &Path,
    config: Arc<Config>,
) -> anyhow::Result<()> {
    let db_holder = DbDropGuard::new();
    let db = db_holder.db();

    let (win_cmd_tx, win_cmd_rx) = channel(32);
    let listener = TcpListener::bind(addr).await?;

    // get TLS acceptor
    let acceptor = get_acceptor(config.clone())?;

    tokio::spawn(async move {
        server::run(listener, db_holder, win_cmd_tx, acceptor, signal::ctrl_c()).await;
    });

    let mut window = SdlWindow::new("viewd", path, win_cmd_rx, db, config)?;

    window.init()?;
    window.handle_event()?;

    Ok(())
}
