use crate::db::{Db, DbDropGuard};
use crate::shutdown::Shutdown;
use crate::window::WindowCommand;
use crate::{Command, Connection};

use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tracing::{debug, error, info};

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
struct Listener {
    db_holder: DbDropGuard,
    /// TCP listener supplied by the `run` caller.
    listener: TcpListener,
    aceptor: TlsAcceptor,
    limit_connections: Arc<Semaphore>,

    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    win_cmd_tx: mpsc::Sender<WindowCommand>,
}

/// Per-connection handler. Reads requests from `connection` and applies the
/// commands.
#[derive(Debug)]
struct Handler {
    db: Db,
    connection: Connection,
    win_cmd_tx: Sender<WindowCommand>,

    shutdown: Shutdown,

    /// Not used directly. Instead, when `Handler` is dropped...?
    _shutdown_complete: mpsc::Sender<()>,
}

const MAX_CONNECTIONS: usize = 250;

pub(crate) async fn run(
    listener: TcpListener,
    aceptor: TlsAcceptor,
    db_holder: DbDropGuard,
    win_cmd_tx: Sender<WindowCommand>,
    shutdown: impl Future,
) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    // Initialize the listener state
    let mut server = Listener {
        listener,
        aceptor,
        db_holder,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
        win_cmd_tx: win_cmd_tx.clone(),
    };

    tokio::select! {
        res = server.run() => {
            // If an error is received here, accepting connections from the TCP
            // listener failed multiple times and the server is giving up and
            // shutting down.
            //
            // Errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            // The shutdown signal has been received.
            info!("shutting down");
            let _ = win_cmd_tx.send(WindowCommand::Quit).await;
        }
    }
    // Extract the `shutdown_complete` receiver and transmitter
    // explicitly drop `shutdown_transmitter`. This is important, as the
    // `.await` below would otherwise never complete.
    let Listener {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    // When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
    // receive the shutdown signal and can exit
    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_complete_tx);

    // Wait for all active connections to finish processing. As the `Sender`
    // handle held by the listener has been dropped above, the only remaining
    // `Sender` instances are held by connection handler tasks. When those drop,
    // the `mpsc` channel will close and `recv()` will return `None`.
    let _ = shutdown_complete_rx.recv().await;
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
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
            let mut handler = Handler {
                // TODO maybe we will do something similar for command exec
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
                    match self.aceptor.accept(socket).await {
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

impl Handler {
    /// Process a single connection.
    // #[instrument(skip(self))]
    async fn run(&mut self) -> crate::Result<()> {
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
