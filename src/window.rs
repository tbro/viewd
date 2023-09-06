use std::fmt;

/// String used as a db key for the image currently displayed
pub const DISPLAY_PATH: &str = "display_path";

/// Possible commands to execute on the Server, either
/// Navigating the files under `--path` or executing comands
/// on the SDL window.
#[derive(Debug, Clone, Copy)]
pub enum WindowCommand {
    /// Go back to the last image
    Prev,
    /// Advance by one image
    Next,
    /// Toggle fullscreen
    Fullscreen,
    /// Toggle fullscreen
    Rotate,
    /// Exit Window control loop
    Quit,
}

impl WindowCommand {
    /// returns WindowCommand for string
    pub(crate) fn from_str(cmd_name: &str) -> crate::Result<WindowCommand> {
        let cmd = match cmd_name.to_lowercase().as_str() {
            "next" => WindowCommand::Next,
            "prev" => WindowCommand::Prev,
            "fullscreen" => WindowCommand::Fullscreen,
            "rotate" => WindowCommand::Rotate,
            &_ => todo!(),
        };

        Ok(cmd)
    }
}

impl fmt::Display for WindowCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Next => write!(f, "Next"),
            Self::Prev => write!(f, "Previous"),
            Self::Fullscreen => write!(f, "Fullscren"),
            Self::Rotate => write!(f, "Rotate"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}
