use clap::Parser;
use crossterm::terminal::{Clear, ClearType, SetTitle};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::io::{self, Write};
use std::time::Duration;
use std::{fmt, str};

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor::MoveTo,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute,
    style::{self, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use viewd::{clients::Client, window::DISPLAY_PATH, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[clap(name = "viewd-tui", version, author, about = "Viewd Terminal UI")]
struct Cli {
    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
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
    /// Event Loop to map keyboard events to TCP commands. Image
    /// currently displayed in the terminal will also be updated
    /// every `delay`.
    async fn handle_events(&mut self) -> viewd::Result<()> {
        loop {
            let mut delay = Delay::new(self.wait).fuse();
            let mut event = self.reader.next().fuse();

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
                     Some(Err(e)) => eprintln!("Error: {:?}\r", e),
                     None => break Ok(()),
                     Some(Ok(event)) => {
                         if let Event::Key(KeyEvent {
                             code, modifiers, ..
                         }) = event
                         {
                             // handle Ctrl-C
                             if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                                 break Ok(());
                             }
                             // handle Command Events
                             match code {
                                 // ESC or 'q' will also end the loop
                                 KeyCode::Esc | KeyCode::Char('q') => break Ok(()),
                                 KeyCode::Char('f') => self.exec(ServerCommand::Fullscreen).await?,
                                 KeyCode::Char('r') => self.exec(ServerCommand::Rotate).await?,
                                 KeyCode::Right => self.exec(ServerCommand::Next).await?,
                                 KeyCode::Left => self.exec(ServerCommand::Prev).await?,
                                 KeyCode::Char(' ') | KeyCode::Char('p') => {
                                     self.exec(ServerCommand::Pageant).await?
                                 }
                                 _ => println!("Unhandled Event::{:?}\r", event),
                             }
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

    // Get the remote address to connect to
    let addr = format!("{}:{}", cli.host, cli.port);

    // Establish a connection
    let client = Client::connect(&addr).await?;

    enable_raw_mode()?;

    let mut tui = Tui::new(client);
    tui.set_title()?;

    if let Err(e) = tui.handle_events().await {
        println!("Error: {:?}\r", e);
    }

    disable_raw_mode()?;

    Ok(())
}
