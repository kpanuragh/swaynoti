use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::notification::Urgency;

/// Sound player with urgency-based sound support
pub struct SoundPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    config: Arc<RwLock<Config>>,
}

impl SoundPlayer {
    pub fn new(config: Arc<RwLock<Config>>) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        info!("Sound player initialized");

        Ok(Self {
            _stream: stream,
            stream_handle,
            config,
        })
    }

    /// Play sound for a notification based on urgency
    pub fn play_for_urgency(&self, urgency: Urgency) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read();

        if !config.sound.enabled {
            debug!("Sound is disabled");
            return Ok(());
        }

        // Get sound file for urgency
        let sound_path = match urgency {
            Urgency::Low => config.sound.sound_low.as_ref(),
            Urgency::Normal => config.sound.sound_normal.as_ref(),
            Urgency::Critical => config.sound.sound_critical.as_ref(),
        };

        // Fall back to default sound
        let sound_path = sound_path.or(config.sound.default_sound.as_ref());

        if let Some(path) = sound_path {
            self.play_file(path)?;
        } else {
            // Try to play freedesktop theme sound based on urgency
            let sound_name = match urgency {
                Urgency::Low => "message",
                Urgency::Normal => "message-new-instant",
                Urgency::Critical => "dialog-warning",
            };
            self.play_sound_name(sound_name)?;
        }

        Ok(())
    }

    /// Play a specific sound file
    pub fn play_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        // Expand ~ in path
        let path = if path.starts_with("~") {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(path.to_string_lossy().replacen("~", &home, 1))
        } else {
            path.to_path_buf()
        };

        debug!("Playing sound file: {}", path.display());

        if !path.exists() {
            warn!("Sound file not found: {}", path.display());
            return Ok(());
        }

        let file = File::open(&path)?;
        let source = Decoder::new(BufReader::new(file))?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.detach(); // Let it play to completion

        Ok(())
    }

    /// Play a sound by name from freedesktop sound theme
    pub fn play_sound_name(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Try common locations for sound themes
        let xdg_data_home = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.local/share", home)
        });

        let search_paths = [
            format!("{}/sounds", xdg_data_home),
            "/usr/share/sounds".to_string(),
            "/usr/local/share/sounds".to_string(),
        ];

        let extensions = ["oga", "ogg", "wav", "mp3"];
        let themes = ["freedesktop", "default", ""];

        for base_path in &search_paths {
            for theme in &themes {
                // Try stereo directory
                for ext in &extensions {
                    let path = if theme.is_empty() {
                        format!("{}/{}.{}", base_path, name, ext)
                    } else {
                        format!("{}/{}/stereo/{}.{}", base_path, theme, name, ext)
                    };
                    if Path::new(&path).exists() {
                        return self.play_file(&path);
                    }
                }
            }
        }

        debug!("Sound '{}' not found in any search path", name);
        Ok(())
    }
}

/// Global sound player instance
pub struct SoundService {
    player: Option<SoundPlayer>,
}

impl SoundService {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        let player = SoundPlayer::new(config).ok();
        if player.is_none() {
            warn!("Sound service not available (no audio output)");
        }
        Self { player }
    }

    pub fn play_for_urgency(&self, urgency: Urgency) {
        if let Some(ref player) = self.player {
            if let Err(e) = player.play_for_urgency(urgency) {
                warn!("Failed to play sound: {}", e);
            }
        }
    }

    pub fn play_file<P: AsRef<Path>>(&self, path: P) {
        if let Some(ref player) = self.player {
            if let Err(e) = player.play_file(path) {
                warn!("Failed to play sound file: {}", e);
            }
        }
    }
}
