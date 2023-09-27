use clap::Parser;
use crossterm::terminal::{Clear, ClearType, SetTitle};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::io;
use std::path::PathBuf;
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
    tui::Tui
};

#[derive(Parser, Debug)]
#[clap(name = "viewd-tui", version, author, about = "Viewd Terminal UI")]
struct Cli {
    #[clap(name = "hostname", long)]
    host: Option<String>,
    #[clap(long)]
    port: Option<u16>,
    #[clap(long, short, default_value = "config/client/example.toml")]
    config: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> viewd::Result<()> {
    // Enable logging
    tracing_subscriber::fmt::try_init()?;

    // Parse command line arguments
    let cli = Cli::parse();
    let config = Config::new(cli.config.as_path())?;
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
    // shutdown TcpStream
    tui.shutdown().await?;

    Ok(())
}
