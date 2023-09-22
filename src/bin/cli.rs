use tokio_rustls::rustls;
use viewd::{clients::Client, window::DISPLAY_PATH, DEFAULT_PORT};

use clap::{Parser, Subcommand};

use std::str;

#[derive(Parser, Debug)]
#[clap(name = "viewd-cli", version, author, about = "Issue Viewd Commands")]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Get the value of key.
    Get,
    Rotate,
    Fullscreen,
    Pageant,
    Next,
    Prev,
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
    let mut root_cert_store = rustls::RootCertStore::empty();
    let mut client = Client::connect(&addr, root_cert_store).await?;

    // Process the requested command
    // Set takes key, value, but currently only key is used.
    match cli.command {
        Command::Next => {
            client.set("next", vec![].into()).await?;
            println!("OK");
        }
        Command::Prev => {
            client.set("prev", vec![].into()).await?;
            println!("OK");
        }
        Command::Get => {
            let s = DISPLAY_PATH;
            if let Some(value) = client.get(s).await? {
                if let Ok(string) = str::from_utf8(&value) {
                    println!("\"{}\"", string);
                } else {
                    println!("{:?}", value);
                }
            } else {
                println!("(nil)");
            }
        }
        Command::Rotate => {
            client.set("rotate", vec![].into()).await?;
            println!("OK");
        }
        Command::Fullscreen => {
            client.set("fullscreen", vec![].into()).await?;
            println!("OK");
        }
        Command::Pageant => {
            client.set("pageant", vec![].into()).await?;
            println!("OK");
        }
    }

    Ok(())
}
