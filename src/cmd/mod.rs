mod get;
pub use get::Get;

mod set;
pub use set::Set;

mod unknown;

pub use unknown::Unknown;

use crate::{Connection, Db, Frame, Parse, Shutdown, WindowCommand};
use tokio::sync::mpsc::Sender;

/// Enumeration of supported Viewed commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Unknown(Unknown),
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        // commands are just simple strings

        let mut parse = Parse::new(frame)?;
        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        parse.finish()?;
        // The command has been successfully parsed
        Ok(command)
    }

    /// Apply the command to the SDL_Window by transmitting it back
    /// through mpsc channel.
    pub(crate) async fn apply(
        self,
        db: &Db,
        tx: Sender<WindowCommand>,
        dst: &mut Connection,
        _shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use Command::*;
        match self {
            Get(cmd) => cmd.apply(db, dst).await,
            Set(cmd) => cmd.apply(tx, dst).await,
            Unknown(cmd) => cmd.apply(dst).await,
        }
    }
}
