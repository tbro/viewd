use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Some methods to move back and forth in a vec of Paths
#[derive(Debug, Clone)]
pub struct PathCursor {
    paths: Vec<PathBuf>,
    /// current index position. it will be None until next() is called
    index: Option<usize>,
    len: usize,
}

impl PathCursor {
    /// initialized a PathCursor from a vec of PathBufs
    pub fn new(items: Vec<PathBuf>) -> Self {
        let index = None;
        let len = items.len();
        Self {
            paths: items,
            index,
            len,
        }
    }
    /// get current index then advance
    pub fn next(&mut self) -> Option<PathBuf> {
        // if not None use the index, else set it to 0
        let index = self.index.map_or_else(|| 0, |i| i);

        // if get returns an item, it must be a valid index. Otherwise cycle
        // get the first image and set the index for the next call to 1
        let path = if let Some(path) = self.paths.get(index) {
            self.index = Some(index + 1);
            Some(path)
        } else {
            self.index = Some(1);
            self.paths.first()
        };
        path.map(|s| s.to_path_buf())
    }
    /// get previous
    pub fn prev(&mut self) -> Option<&PathBuf> {
        // if not None use the index, else set it to 0
        let mut index = self.index.map_or_else(|| 0, |i| i);
        let path = if index == 0 {
            index = self.len - 1;
            self.paths.last()
        } else {
            index -= 1;
            self.paths.get(index)
        };
        self.index = Some(index);
        path
    }
    /// remove
    pub fn remove(&mut self) -> Option<PathBuf> {
        if let Some(index) = self.index {
            self.len -= 1;
            let p = self.paths.remove(index);
            Some(p)
        } else {
            None
        }
    }

    /// Import all the files under given dir path, performing some sanity checks.
    pub fn import_files(path: &Path) -> Result<Self> {
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
        Ok(Self::new(paths))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn get_paths() -> Vec<PathBuf> {
        let v = vec![
            Path::new("./foo/bar.txt"),
            Path::new("./bim/bam.txt"),
            Path::new("./bar/foo.txt"),
        ];
        v.iter().map(|i| i.to_path_buf()).collect()
    }

    #[test]
    fn test_cursor_next() -> Result<()> {
        let p = get_paths();
        let mut v = PathCursor::new(p);
        assert_eq!(v.next(), Some(Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.next(), Some(Path::new("./bim/bam.txt").to_path_buf()));
        assert_eq!(v.next(), Some(Path::new("./bar/foo.txt").to_path_buf()));
        assert_eq!(v.next(), Some(Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.next(), Some(Path::new("./bim/bam.txt").to_path_buf()));
        Ok(())
    }
    #[test]
    fn test_cursor_prev() -> Result<()> {
        let p = get_paths();
        let mut v = PathCursor::new(p);
        assert_eq!(v.prev(), Some(&Path::new("./bar/foo.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./bim/bam.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./bar/foo.txt").to_path_buf()));
        Ok(())
    }
    #[test]
    fn test_walkdir() -> Result<()> {
	// unnecessary test for personal sanity
	let mut glob = WalkDir::new("./").into_iter().filter_map(|e| e.ok());
	let x = glob.find(|e|e.file_name() == "Cargo.toml").unwrap();
	assert_eq!("Cargo.toml", x.file_name());
        Ok(())
    }
}
