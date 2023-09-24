use std::time::{Duration, Instant};

/// A type to represent pageant mode
#[derive(Debug, Copy, Clone)]
pub struct PageantMode {
    /// milliseconds each image will remain visible
    timeout: u64,
    /// time of last update
    instant: Option<Instant>,
}

impl PageantMode {
    pub fn new(timeout: u64) -> Self {
        Self {
            timeout,
            instant: None,
        }
    }
    /// Toggle Option, self.instant is `Some`, we are in pageant mode.
    /// This will be called from `should_update`.
    pub fn toggle(&mut self) {
        if let Some(_instant) = self.instant {
            self.instant = None;
        } else {
            self.instant = Some(Instant::now());
        }
    }
    pub fn set_instant(&mut self) {
        self.instant = Some(Instant::now());
    }
    /// Check if we have an instant and if so if timeout value has elapsed.
    pub fn should_update(&self) -> bool {
        if let Some(instant) = self.instant {
            Instant::now() - instant >= Duration::from_millis(self.timeout)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn toggle_pageant() -> Result<()> {
        let mut pageant = PageantMode::new(1000);
        assert!(pageant.instant.is_none());
        pageant.toggle();
        assert!(pageant.instant.is_some());
        Ok(())
    }
}
