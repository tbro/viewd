use crate::db::DbDropGuard;
use crate::shutdown::Shutdown;
use crate::window::WindowCommand;
use crate::Connection;

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tracing::{error, info};

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
pub struct Listener {
    pub db_holder: DbDropGuard,
    /// TCP listener supplied by the `run` caller.
    pub listener: TcpListener,
    pub acceptor: TlsAcceptor,
    pub limit_connections: Arc<Semaphore>,

    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown_complete_tx: mpsc::Sender<()>,
    pub win_cmd_tx: mpsc::Sender<WindowCommand>,
}

impl Listener {
    pub async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");
        loop {
            // Wait for a permit to become available
            //
            // `acquire_owned` returns a permit that is bound to the semaphore.
            // When the permit value is dropped, it is automatically returned
            // to the semaphore.
            //
            // `acquire_owned()` returns `Err` when the semaphore has been
            // closed. We don't ever close the semaphore, so `unwrap()` is safe.
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            // Accept a new socket. This will attempt to perform error handling.
            // The `accept` method internally attempts to recover errors, so an
            // error here is non-recoverable.
            let socket = self.accept().await?;

            // Create the necessary per-connection handler state.
            let mut handler = super::Handler {
                // Get a handle to the shared database.
                db: self.db_holder.db(),
                win_cmd_tx: self.win_cmd_tx.clone(),

                // Initialize the connection state. This allocates read/write
                // buffers to perform frame parsing.
                connection: Connection::new(socket.into()),

                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),

                // Notifies the receiver half once all clones are
                // dropped.
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            // Spawn a new task to process the connections. Tokio tasks are like
            // asynchronous green threads and are executed concurrently.
            tokio::spawn(async move {
                // Process the connection. If an error is encountered, log it.
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
                // Move the permit into the task and drop it after completion.
                // This returns the permit back to the semaphore.
                drop(permit);
            });
        }
    }

    /// Accept an inbound connection.
    ///
    /// TCP connection errors and TLS acceptance errors are handled by
    /// backing off and retrying. An exponential backoff strategy is
    /// used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each
    /// subsequent failure doubles the wait time. If accepting fails
    /// on the 6th try after waiting for 64 seconds, then this
    /// function returns with an error. The nested structure means
    /// there are two sequential and identical backoff procdures, the
    /// first for the TCP connection and the second for TLS.
    async fn accept(&mut self) -> crate::Result<TlsStream<TcpStream>> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the TCP accept operation. If a socket is successfully
            // accepted, it to TLS. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    // Perform the TLS accept operation. If a socket is successfully
                    // accepted, return it. Otherwise, save the error.
                    match self.acceptor.accept(socket).await {
                        Ok(socket) => {
                            return Ok(socket);
                        }
                        Err(err) => {
                            if backoff > 64 {
                                // Accept has failed too many times. Return the error.
                                return Err(err.into());
                            }
                        }
                    };
                }
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            };
            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}
