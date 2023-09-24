use clap::Parser;
use crossterm::terminal::{Clear, ClearType, SetTitle};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::io;
use std::time::Duration;
use std::{fmt, str};
use tracing::debug;

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor::MoveTo,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute,
    style::{self, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use viewd::{
    clients::{Client, Config},
    window::DISPLAY_PATH,
};

#[derive(Parser, Debug)]
#[clap(name = "viewd-tui", version, author, about = "Viewd Terminal UI")]
struct Cli {
    #[clap(name = "hostname", long)]
    host: Option<String>,
    #[clap(long)]
    port: Option<u16>,
}

/// Enumeration of commands to send to Server
// TODO instead of wrapping `client.set` we could have an exec network
// command which uses a similar enum
enum ServerCommand {
    Fullscreen,
    Rotate,
    Prev,
    Next,
    Pageant,
}

impl fmt::Display for ServerCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Fullscreen => write!(f, "fullscreen"),
            Self::Rotate => write!(f, "rotate"),
            Self::Pageant => write!(f, "pageant"),
            Self::Next => write!(f, "next"),
            Self::Prev => write!(f, "prev"),
        }
    }
}

struct Shutdown(bool);
/// Struct to hold Terminual UI
struct Tui {
    /// Time to wait between ui updates
    wait: Duration,
    /// viewd Client
    client: Client,
    /// EventStream reader
    reader: EventStream,
    /// Window title,
    title: String,
}

impl Tui {
    pub fn new(client: Client) -> Self {
        let reader = EventStream::new();
        let wait = Duration::from_millis(1_000);
        let title = String::from("viewd-tui");
        Self {
            wait,
            client,
            reader,
            title,
        }
    }
    /// Wrap the TCP command setter
    async fn exec(&mut self, cmd: ServerCommand) -> viewd::Result<()> {
        self.client.set(&cmd.to_string(), vec![].into()).await
    }
    /// Update currently displayed image name
    fn update(&self, s: &str) -> io::Result<()> {
        execute!(
            io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            style::Print(s.magenta())
        )?;
        Ok(())
    }
    /// Send Tls Stream the shutdown signal
    async fn shutdown(self) -> viewd::Result<()> {
        self.client.shutdown().await
    }
    /// Handle keyboard input
    async fn handle_keycode(&mut self, event: Event) -> viewd::Result<Shutdown> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            // handle Ctrl-C
            if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                return Ok(Shutdown(true));
            }

            match code {
                // ESC or 'q' will also end the loop
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Ok(Shutdown(true));
                }
                KeyCode::Char('f') => self.exec(ServerCommand::Fullscreen).await?,
                KeyCode::Char('r') => self.exec(ServerCommand::Rotate).await?,
                KeyCode::Right => self.exec(ServerCommand::Next).await?,
                KeyCode::Left => self.exec(ServerCommand::Prev).await?,
                KeyCode::Char(' ') | KeyCode::Char('p') => {
                    self.exec(ServerCommand::Pageant).await?
                }
                _ => debug!("Unhandled Event:: {:?}", code),
            };
        }
        Ok(Shutdown(false))
    }
    /// Event Loop to map keyboard events to TCP commands. Image
    /// currently displayed in the terminal will also be updated
    /// every `delay`.
    #[rustfmt::skip]
    async fn handle_events(&mut self) -> viewd::Result<()> {
        loop {
            let mut delay = Delay::new(self.wait).fuse();
            let mut event = self.reader.next().fuse();

            // select on `get`s to server and polling for keyboard input
            select! {
            _ = delay => {
                    let s = DISPLAY_PATH;
                    if let Some(value) = self.client.get(s).await? {
                        if let Ok(s) = str::from_utf8(&value) {
			    self.update(s)?
                         }
                     }
                 }
                 maybe_event = event => {
                     match maybe_event {
                         Some(Err(e)) => debug!("Error: {:?}\r", e),
                         None => break Ok(()),
                         Some(Ok(event)) => {
                             // handle Command Events
			     // if Shutdown was set to true, break
			     if self.handle_keycode(event).await?.0 {
				 break Ok(());
			     }
                         }
                     }
		 }
	    }
        }
    }
    /// Set terminal window title and clear the terminal
    fn set_title(&self) -> io::Result<()> {
        execute!(
            io::stdout(),
            SetTitle(self.title.as_str()),
            Clear(ClearType::All),
        )?;
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> viewd::Result<()> {
    // Enable logging
    tracing_subscriber::fmt::try_init()?;

    // Parse command line arguments
    let cli = Cli::parse();
    let config = Config::new()?;
    let con_config = config.clone();
    let host = cli.host.unwrap_or(config.host.to_string());
    let port = cli.port.unwrap_or(config.port);

    // Establish a connection
    let client = Client::connect(&host, port, con_config).await?;

    enable_raw_mode()?;

    let mut tui = Tui::new(client);
    tui.set_title()?;

    if let Err(e) = tui.handle_events().await {
        debug!("Error: {:?}\r", e);
    }

    disable_raw_mode()?;
    // calls shutdown on the TcpStream
    tui.shutdown().await?;

    Ok(())
}
