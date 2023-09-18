use crate::cmd::Parse;
use crate::window::WindowCommand;
use crate::{Connection, Frame};

use bytes::Bytes;
use tokio::sync::mpsc::Sender;

use tracing::{debug, instrument};

/// We only store toggles
#[derive(Debug)]
pub struct Set {
    /// Toggle to switch
    key: String,
    value: Bytes,
}

impl Set {
    /// Create a new `Set` command which toggles `key`.
    pub fn new(key: impl ToString, value: Bytes) -> Set {
        Set {
            key: key.to_string(),
            value,
        }
    }

    /// Get the key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Parse a `Set` instance from a received frame.
    ///
    /// The `Parse` argument provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from
    /// the socket.
    ///
    /// The `SET` string has already been consumed.
    ///
    /// # Returns
    ///
    /// Returns the `Set` value on success. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing at least 3 entries.
    ///
    /// ```text
    /// SET key value
    /// ```
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Set> {
        // Read the key to set. This is a required field
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        Ok(Set { key, value })
    }

    /// Transmit the `Set` command to the `SdlWindow` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, tx, dst))]
    pub(crate) async fn apply(
        self,
        tx: Sender<WindowCommand>,
        dst: &mut Connection,
    ) -> crate::Result<()> {
        // get WindowCommand variant for command string
        // and transmit it to the Window
        let cmd = WindowCommand::from_str(&self.key)?;
        tx.send(cmd).await?;

        // Create a success response and write it to `dst`.
        let response = Frame::Simple("OK".to_string());
        debug!(?response);

        dst.write_frame(&response).await?;

        Ok(())
    }

    /// Converts the command into an equivalent `Frame`.
    ///
    /// This is called by the client when encoding a `Set` command to send to
    /// the server.
    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("set".as_bytes()));
        frame.push_bulk(Bytes::from(self.key.into_bytes()));
        frame.push_bulk(self.value);
        frame
    }
}
