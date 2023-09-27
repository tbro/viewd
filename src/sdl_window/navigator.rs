use anyhow::anyhow;
use anyhow::Result;
use sdl2::image::LoadSurface;
use sdl2::surface::Surface;
use walkdir::WalkDir;

use super::cursor::Cursor;
use std::path::Path;
use std::path::PathBuf;

/// Navigator holds the list of images and methods to move through
/// them. It wraps cursor to provide a facade for simplifying the
/// cursor API. `image` holds the path of the file currently under
/// the cursor.
#[derive(Debug, Clone)]
pub struct Navigator {
    cursor: Cursor<PathBuf>,
    pub image: PathBuf,
}

impl Navigator {
    /// initialized the cursor and checks does a basic check that
    /// cursor holds at least one path.
    pub fn new(path: &Path) -> Result<Self> {
        let mut cursor = Navigator::import_files(path)?;
        let image = cursor.next().ok_or(anyhow!("no image found"))?;
        let n = Self { cursor, image };
        Ok(n)
    }
    /// Advance the cursor and return current. Test that path is a
    /// supported image by loading it in a throw away surface. If it
    /// is not supported, and cursor is not empty, call again. When we
    /// encounter unsupported files, they are removed from the
    /// cursor. Since it is possible that we are opperting on a collection
    /// of all unsupported files, removing them will eventually result in an
    /// empty collection and return None.
    pub fn next(&mut self) -> Option<PathBuf> {
        let previous = &self.image;
        let path = self.cursor.next()?;
        // if cursor is empty, return None
        if self.cursor.is_empty() {
            return None;
        }
        // if file is supported and cursor has moved return path
        let maybe_image = Surface::from_file(&path);
        if maybe_image.is_ok() && &path != previous {
            self.image = path.to_path_buf();
            return Some(path);
        }
        if maybe_image.is_err() {
            self.cursor.remove();
        }
        // if above conditions are not met, call again
        self.next()
    }
    /// Opposite of next. See [`Self.next()`].
    pub fn prev(&mut self) -> Option<PathBuf> {
        let previous = &self.image;
        let path = self.cursor.prev()?;
        // if cursor is empty, return None
        if self.cursor.is_empty() {
            return None;
        }
        let maybe_image = Surface::from_file(&path);
        // if file is supported and cursor has moved return path
        if maybe_image.is_ok() && &path != previous {
            self.image = path.to_path_buf();
            return Some(path);
        }
        // remove unsupported files from collection
        if maybe_image.is_err() {
            self.cursor.remove();
        }
        // if above conditions are not met, call again
        self.prev()
    }
    /// Import all the files under given dir path, performing some sanity checks.
    pub fn import_files(path: &Path) -> Result<Cursor<PathBuf>> {
        use rayon::prelude::*;
        let mut paths = WalkDir::new(path)
            .into_iter()
            .par_bridge()
            // ignore i/o errors
            .filter_map(|e| e.ok())
            // filter out directories
            .filter(|x| !x.file_type().is_dir())
            .map(|x| x.into_path())
            .collect::<Vec<PathBuf>>();

        if paths.is_empty() {
            return Err(anyhow!("no files found in image directory"));
        }
        paths.par_sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(Cursor::new(paths))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walkdir() -> Result<()> {
        // unnecessary test for personal sanity
        let mut glob = WalkDir::new("./").into_iter().filter_map(|e| e.ok());
        let x = glob.find(|e| e.file_name() == "Cargo.toml").unwrap();
        assert_eq!("Cargo.toml", x.file_name());
        Ok(())
    }
}
