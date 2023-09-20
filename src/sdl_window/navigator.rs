use anyhow::Result;
use anyhow::bail;

use std::path::Path;
use std::path::PathBuf;

use super::cursor::PathCursor;

/// Navigator holds the list of images and methods to move through
/// them. It wraps cursor to provide a facade for simplifying the
/// cursor API. `image` holds the path of the file currently under
/// the cursor.
#[derive(Debug, Clone)]
pub struct Navigator {
    cursor: PathCursor,
    pub image: PathBuf,
}

impl Navigator {
    /// initialized the cursor and checks does a basic check that
    /// cursor holds at least one path.
    pub fn new(path: &Path) -> Result<Self> {
        let mut cursor = PathCursor::import_files(path)?;
        let image = if let Some(path) = cursor.next() {
            path.to_path_buf()
        } else {
            return bail!("no images found");
        };
        let n = Self { cursor, image };
        Ok(n)
    }
    /// advance the cursor and return current
    pub fn next(&mut self) -> Option<PathBuf> {
        let path = self.cursor.next()?;
        self.image = path.to_path_buf();
        Some(path)
    }
    /// opposite of next
    pub fn prev(&mut self) -> Option<&Path> {
        let path = self.cursor.prev()?;
        self.image = path.to_path_buf();
        Some(path)
    }
}
