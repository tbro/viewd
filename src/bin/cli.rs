use clap::{Parser, Subcommand};
use std::{path::PathBuf, str};
use viewd::{
    clients::{Client, Config},
    window::DISPLAY_PATH,
};

#[derive(Parser, Debug)]
#[clap(name = "viewd-cli", version, author, about = "Issue Viewd Commands")]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long)]
    host: Option<String>,

    #[clap(long)]
    port: Option<u16>,

    #[clap(long, short, default_value = "config/client/example.toml")]
    config: PathBuf,
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

    // Establish a connection
    let config = Config::new(cli.config.as_path())?;
    let con_config = config.clone();

    let host = cli.host.unwrap_or(config.host.to_string());
    let port = cli.port.unwrap_or(config.port);

    let mut client = Client::connect(&host, port, con_config).await?;

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
