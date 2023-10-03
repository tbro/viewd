use crate::db::Db;
use crate::shutdown::Shutdown;
use crate::window::WindowCommand;
use crate::{Command, Connection};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use tracing::debug;

/// Per-connection handler. Reads requests from `connection` and applies the
/// commands.
#[derive(Debug)]
pub(crate) struct Handler {
    pub db: Db,
    pub connection: Connection,
    pub win_cmd_tx: Sender<WindowCommand>,

    pub shutdown: Shutdown,

    /// Not used directly. Instead, when `Handler` is dropped...?
    pub _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    /// Process a single connection.
    // #[instrument(skip(self))]
    pub async fn run(&mut self) -> crate::Result<()> {
        // As long as the shutdown signal has not been received, try to read a
        // new request frame.
        while !self.shutdown.is_shutdown() {
            // While reading a request frame, also listen for the shutdown
            // signal.
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
            };

            // If `None` is returned from `read_frame()` then the peer closed
            // the socket. There is no further work to do and the task can be
            // terminated.
            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            debug!(?frame);
            let cmd = Command::from_frame(frame)?;

            // Perform the work needed to apply the command. Set
            // Commands are passed and transmitted to SDL Window over
            // Mpsc channel. Currently only Window mutates the database
            // but it might make sense in the future to add commands
            // that mutate the db directly.
            cmd.apply(
                &self.db,
                self.win_cmd_tx.clone(),
                &mut self.connection,
                &mut self.shutdown,
            )
            .await?;
        }

        Ok(())
    }
}
