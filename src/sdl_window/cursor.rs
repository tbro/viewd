/// Some methods to move back and forth in a vec
#[derive(Debug, Clone)]
pub struct Cursor<T> {
    items: Vec<T>,
    /// current index position. it will be None until next() is called
    index: Option<usize>,
    len: usize,
}

impl<T: std::clone::Clone> Cursor<T> {
    /// initialized a PathCursor from a vec of PathBufs
    pub fn new(items: Vec<T>) -> Self {
        let index = None;
        let len = items.len();
        Self {
            items,
            index,
            len,
        }
    }
    /// get current index then advance
    pub fn next(&mut self) -> Option<T> {
        // if not None use the index, else set it to 0
        let index = self.index.map_or_else(|| 0, |i| i);

        // if get returns an item, it must be a valid index. Otherwise cycle
        // get the first image and set the index for the next call to 1
        let item = if let Some(path) = self.items.get(index) {
            self.index = Some(index + 1);
            Some(path)
        } else {
            self.index = Some(1);
            self.items.first()
        };
	item.cloned()
    }
    /// get previous
    pub fn prev(&mut self) -> Option<T> {
        // if not None use the index, else set it to 0
        let mut index = self.index.map_or_else(|| 0, |i| i);
        let item = if index == 0 {
            index = self.len - 1;
            self.items.last()
        } else {
            index -= 1;
            self.items.get(index)
        };
        self.index = Some(index);
	item.cloned()
    }
    /// check if Vec is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    /// remove
    pub fn remove(&mut self) -> Option<T> {
        if let Some(index) = self.index {
            self.len -= 1;
            let p = self.items.remove(index);
            Some(p)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_cursor_next() -> Result<()> {
        let v = vec![1,2,3];
        let mut v = Cursor::new(v);
        assert_eq!(v.next(), Some(1));
        assert_eq!(v.next(), Some(2));
        assert_eq!(v.next(), Some(3));
        assert_eq!(v.next(), Some(1));
        assert_eq!(v.next(), Some(2));
        Ok(())
    }
    #[test]
    fn test_cursor_prev() -> Result<()> {
        let v = vec![1,2,3];
        let mut v = Cursor::new(v);
        assert_eq!(v.prev(), Some(3));
        assert_eq!(v.prev(), Some(2));
        assert_eq!(v.prev(), Some(1));
        assert_eq!(v.prev(), Some(3));
        assert_eq!(v.prev(), Some(2));
        Ok(())
    }
}
