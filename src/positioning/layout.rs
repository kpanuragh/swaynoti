use parking_lot::RwLock;
use std::sync::Arc;

use crate::config::Config;

/// Manages notification layout and positioning
pub struct LayoutManager {
    config: Arc<RwLock<Config>>,
}

impl LayoutManager {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
    }

    /// Calculate the position for a notification at the given index
    pub fn calculate_position(&self, _index: usize) -> (i32, i32, i32, i32) {
        let config = self.config.read();
        let margin = &config.positioning.margin;

        // This returns (top, right, bottom, left) margins
        // The actual position depends on anchor
        (margin.top, margin.right, margin.bottom, margin.left)
    }
}
