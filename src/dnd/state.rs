use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

/// Do Not Disturb state
pub struct DndState {
    enabled: AtomicBool,
    /// Whether DND was enabled manually (not by schedule)
    manual: AtomicBool,
}

impl Default for DndState {
    fn default() -> Self {
        Self::new()
    }
}

impl DndState {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            manual: AtomicBool::new(false),
        }
    }

    /// Check if DND is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Enable DND mode
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
        self.manual.store(true, Ordering::SeqCst);
        info!("Do Not Disturb enabled");
    }

    /// Disable DND mode
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        self.manual.store(false, Ordering::SeqCst);
        info!("Do Not Disturb disabled");
    }

    /// Toggle DND mode
    pub fn toggle(&self) {
        let current = self.enabled.load(Ordering::SeqCst);
        self.enabled.store(!current, Ordering::SeqCst);
        self.manual.store(!current, Ordering::SeqCst);
        info!("Do Not Disturb toggled to {}", !current);
    }

    /// Enable DND from schedule (won't override manual setting)
    pub fn enable_scheduled(&self) {
        if !self.manual.load(Ordering::SeqCst) {
            self.enabled.store(true, Ordering::SeqCst);
            info!("Do Not Disturb enabled by schedule");
        }
    }

    /// Disable DND from schedule (won't override manual setting)
    pub fn disable_scheduled(&self) {
        if !self.manual.load(Ordering::SeqCst) {
            self.enabled.store(false, Ordering::SeqCst);
            info!("Do Not Disturb disabled by schedule");
        }
    }

    /// Check if DND was enabled manually
    pub fn is_manual(&self) -> bool {
        self.manual.load(Ordering::SeqCst)
    }
}
