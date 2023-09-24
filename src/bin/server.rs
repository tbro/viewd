//! viewd server.
//!
//! This file is the entry point for the server implemented in the library. It
//! performs command line parsing and passes the arguments on to
//! `viewd::server`.
//!
//! The `clap` crate is used for parsing arguments.

use clap::Parser;
use std::path::PathBuf;
use viewd::control;
use viewd::serve::{get_acceptor, Config};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    set_up_logging()?;

    let cli = Cli::parse();
    let config = Config::new()?;

    // get the TLS acceptor
    let acceptor = get_acceptor(config.clone())?;
    let port = cli.port.unwrap_or(config.port);
    let path = cli
        .path
        .unwrap_or_else(|| config.path.as_path().to_path_buf());

    control::run(&format!("127.0.0.1:{}", port), &path, config, acceptor).await?;
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "viewd-server", version, author, about = "A Viewd Server")]
pub struct Cli {
    #[clap(long)]
    port: Option<u16>,
    #[clap(long)]
    path: Option<PathBuf>,
}

fn set_up_logging() -> anyhow::Result<()> {
    // See https://docs.rs/tracing for more info
    tracing_subscriber::fmt::try_init().map_err(|e| anyhow::anyhow!(e))
}
