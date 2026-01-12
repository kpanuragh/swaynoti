use std::path::Path;
use std::sync::Arc;

use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{CssProvider, StyleContext};
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::config::defaults::DEFAULT_CSS;

/// Manages CSS styling for the application
pub struct StyleManager {
    provider: CssProvider,
    config: Arc<RwLock<Config>>,
}

impl StyleManager {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        let provider = CssProvider::new();

        Self { provider, config }
    }

    /// Load and apply CSS styles
    pub fn load_styles(&self) {
        let config = self.config.read();

        // Try to load user theme if specified
        if let Some(ref theme_path) = config.appearance.theme {
            let expanded = shellexpand::tilde(&theme_path.to_string_lossy()).to_string();
            let path = Path::new(&expanded);

            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(css) => {
                        info!("Loading custom theme from {:?}", path);
                        self.provider.load_from_string(&css);
                        self.apply_provider();
                        return;
                    }
                    Err(e) => {
                        warn!("Failed to read theme file {:?}: {}", path, e);
                    }
                }
            } else {
                warn!("Theme file not found: {:?}", path);
            }
        }

        // Fall back to default CSS
        debug!("Using default CSS theme");
        self.provider.load_from_string(DEFAULT_CSS);
        self.apply_provider();
    }

    /// Apply the CSS provider to the default display
    fn apply_provider(&self) {
        if let Some(display) = Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &self.provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    /// Reload styles (for hot reload)
    pub fn reload(&self) {
        info!("Reloading CSS styles");
        self.load_styles();
    }
}
