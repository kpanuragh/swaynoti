use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub positioning: PositioningConfig,
    pub timeouts: TimeoutConfig,
    pub history: HistoryConfig,
    pub dnd: DndConfig,
    pub sound: SoundConfig,
    pub ipc: IpcConfig,
    #[serde(default)]
    pub rules: Vec<AppRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Maximum number of visible notifications
    pub max_visible: u32,
    /// Sort order for notifications
    pub sort_order: SortOrder,
    /// Enable markup parsing in body
    pub markup: bool,
    /// Idle threshold in seconds (pause timeouts when idle)
    pub idle_threshold: Option<u64>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_visible: 5,
            sort_order: SortOrder::NewestFirst,
            markup: true,
            idle_threshold: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AppearanceConfig {
    /// Path to custom CSS file
    pub theme: Option<PathBuf>,
    /// Notification width in pixels
    pub width: u32,
    /// Maximum notification height
    pub max_height: u32,
    /// Border radius in pixels
    pub border_radius: u32,
    /// Gap between notifications
    pub gap: u32,
    /// Transparency (0.0 - 1.0)
    pub opacity: f64,
    /// Icon size in pixels
    pub icon_size: u32,
    /// Show app name
    pub show_app_name: bool,
    /// Animation settings
    pub animations: AnimationConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: None,
            width: 350,
            max_height: 200,
            border_radius: 12,
            gap: 8,
            opacity: 0.95,
            icon_size: 48,
            show_app_name: true,
            animations: AnimationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration_ms: u32,
    pub slide_direction: SlideDirection,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_ms: 200,
            slide_direction: SlideDirection::Right,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PositioningConfig {
    /// Screen position anchor
    pub anchor: Anchor,
    /// Layer (overlay, top, bottom, background)
    pub layer: Layer,
    /// Stacking mode
    pub stacking: StackingMode,
    /// Margins from screen edges
    pub margin: MarginConfig,
    /// Monitor selection
    pub monitor: MonitorConfig,
}

impl Default for PositioningConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopRight,
            layer: Layer::Overlay,
            stacking: StackingMode::Vertical,
            margin: MarginConfig::default(),
            monitor: MonitorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MarginConfig {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Default for MarginConfig {
    fn default() -> Self {
        Self {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MonitorConfig {
    /// Which monitor to display on
    pub selection: MonitorSelection,
    /// Specific monitor name (if selection is "named")
    pub name: Option<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            selection: MonitorSelection::Focused,
            name: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TimeoutConfig {
    /// Default timeout in milliseconds
    pub default: i32,
    /// Low urgency timeout
    pub low: i32,
    /// Normal urgency timeout
    pub normal: i32,
    /// Critical urgency timeout (0 = never)
    pub critical: i32,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default: 5000,
            low: 3000,
            normal: 5000,
            critical: 0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HistoryConfig {
    /// Enable history
    pub enabled: bool,
    /// Maximum history entries
    pub max_entries: u32,
    /// History database path
    pub database_path: Option<PathBuf>,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 100,
            database_path: None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DndConfig {
    /// Start time for scheduled DND (HH:MM format)
    pub schedule_start: Option<String>,
    /// End time for scheduled DND
    pub schedule_end: Option<String>,
    /// Days of week for scheduled DND
    pub schedule_days: Vec<Weekday>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    pub default_sound: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct IpcConfig {
    /// Unix socket path
    pub socket_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppRule {
    /// Match criteria
    pub criteria: RuleCriteria,
    /// Actions to apply
    #[serde(default)]
    pub actions: RuleActions,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleCriteria {
    pub app_name: Option<String>,
    pub summary: Option<String>,
    pub body: Option<String>,
    pub urgency: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleActions {
    pub timeout: Option<i32>,
    pub urgency: Option<String>,
    pub anchor: Option<Anchor>,
    pub skip_history: Option<bool>,
    pub skip_sound: Option<bool>,
    pub css_class: Option<String>,
}

// Enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    TopLeft,
    TopCenter,
    #[default]
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
    Background,
    Bottom,
    Top,
    #[default]
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StackingMode {
    #[default]
    Vertical,
    Horizontal,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SortOrder {
    #[default]
    NewestFirst,
    OldestFirst,
    UrgencyDescending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MonitorSelection {
    Primary,
    #[default]
    Focused,
    All,
    Named,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SlideDirection {
    #[default]
    Right,
    Left,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
