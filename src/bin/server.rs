//! viewd server.
//!
//! This file is the entry point for the server implemented in the library. It
//! performs command line parsing and passes the arguments on to
//! `viewd::server`.
//!
//! The `clap` crate is used for parsing arguments.

use std::path::PathBuf;

use viewd::control;
use viewd::DEFAULT_PORT;

use clap::Parser;

#[tokio::main]
pub async fn main() -> viewd::Result<()> {
    set_up_logging()?;

    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);
    let path = cli.path;
    let _ = control::run(&format!("127.0.0.1:{}", port), &path).await;
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "viewd-server", version, author, about = "An Viewd Server")]
struct Cli {
    #[clap(long)]
    port: Option<u16>,
    #[clap(long)]
    path: PathBuf,
}

fn set_up_logging() -> viewd::Result<()> {
    // See https://docs.rs/tracing for more info
    tracing_subscriber::fmt::try_init()
}
