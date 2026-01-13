use std::collections::HashMap;

use parking_lot::RwLock;
use tracing::{debug, info, warn};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedValue, Value};

/// Playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl From<&str> for PlaybackStatus {
    fn from(s: &str) -> Self {
        match s {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
    }
}

/// Media information
#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: Option<String>,
    pub length_us: i64,
    pub position_us: i64,
    pub status: Option<PlaybackStatus>,
    pub player_name: String,
}

/// MPRIS player client
pub struct MprisPlayer {
    connection: Connection,
    current_player: RwLock<Option<String>>,
}

impl MprisPlayer {
    /// Create a new MPRIS player client
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::session()?;
        info!("MPRIS client connected to session bus");

        Ok(Self {
            connection,
            current_player: RwLock::new(None),
        })
    }

    /// Find available media players
    pub fn find_players(&self) -> Vec<String> {
        let proxy = match zbus::blocking::fdo::DBusProxy::new(&self.connection) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create DBus proxy: {}", e);
                return Vec::new();
            }
        };

        match proxy.list_names() {
            Ok(names) => {
                let players: Vec<String> = names
                    .into_iter()
                    .filter(|n| n.as_str().starts_with("org.mpris.MediaPlayer2."))
                    .map(|n| n.to_string())
                    .collect();
                debug!("Found MPRIS players: {:?}", players);
                players
            }
            Err(e) => {
                warn!("Failed to list D-Bus names: {}", e);
                Vec::new()
            }
        }
    }

    /// Get the active player (first playing, or first available)
    pub fn get_active_player(&self) -> Option<String> {
        let players = self.find_players();

        if players.is_empty() {
            return None;
        }

        // Prefer a playing player
        for player in &players {
            if let Some(info) = self.get_media_info(player) {
                if info.status == Some(PlaybackStatus::Playing) {
                    return Some(player.clone());
                }
            }
        }

        // Fall back to first player
        players.into_iter().next()
    }

    /// Get media info from a specific player
    pub fn get_media_info(&self, player_name: &str) -> Option<MediaInfo> {
        let proxy = zbus::blocking::Proxy::new(
            &self.connection,
            player_name,
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        )
        .ok()?;

        let mut info = MediaInfo {
            player_name: player_name
                .strip_prefix("org.mpris.MediaPlayer2.")
                .unwrap_or(player_name)
                .to_string(),
            ..Default::default()
        };

        // Get playback status
        let status_result: Result<String, _> = proxy.get_property("PlaybackStatus");
        if let Ok(status) = status_result {
            info.status = Some(PlaybackStatus::from(status.as_str()));
        }

        // Get metadata
        let metadata_result: Result<HashMap<String, OwnedValue>, _> =
            proxy.get_property("Metadata");
        if let Ok(metadata) = metadata_result {
            // Title
            if let Some(title) = metadata.get("xesam:title") {
                if let Ok(t) = TryInto::<String>::try_into(&**title) {
                    info.title = t;
                }
            }

            // Artist
            if let Some(artist) = metadata.get("xesam:artist") {
                if let Value::Array(arr) = &**artist {
                    let artists: Vec<String> = arr
                        .iter()
                        .filter_map(|v| TryInto::<String>::try_into(v).ok())
                        .collect();
                    info.artist = artists.join(", ");
                }
            }

            // Album
            if let Some(album) = metadata.get("xesam:album") {
                if let Ok(a) = TryInto::<String>::try_into(&**album) {
                    info.album = a;
                }
            }

            // Art URL
            if let Some(art) = metadata.get("mpris:artUrl") {
                if let Ok(url) = TryInto::<String>::try_into(&**art) {
                    info.art_url = Some(url);
                }
            }

            // Length
            if let Some(length) = metadata.get("mpris:length") {
                if let Ok(l) = TryInto::<i64>::try_into(&**length) {
                    info.length_us = l;
                }
            }
        }

        // Get position
        let pos_result: Result<i64, _> = proxy.get_property("Position");
        if let Ok(pos) = pos_result {
            info.position_us = pos;
        }

        Some(info)
    }

    /// Get current media info (from active player)
    pub fn get_current_media(&self) -> Option<MediaInfo> {
        let player = self.get_active_player()?;
        *self.current_player.write() = Some(player.clone());
        self.get_media_info(&player)
    }

    /// Play/Pause toggle
    pub fn play_pause(&self) -> bool {
        if let Some(ref player) = *self.current_player.read() {
            return self.send_command(player, "PlayPause");
        } else if let Some(player) = self.get_active_player() {
            return self.send_command(&player, "PlayPause");
        }
        false
    }

    /// Next track
    pub fn next(&self) -> bool {
        if let Some(ref player) = *self.current_player.read() {
            return self.send_command(player, "Next");
        } else if let Some(player) = self.get_active_player() {
            return self.send_command(&player, "Next");
        }
        false
    }

    /// Previous track
    pub fn previous(&self) -> bool {
        if let Some(ref player) = *self.current_player.read() {
            return self.send_command(player, "Previous");
        } else if let Some(player) = self.get_active_player() {
            return self.send_command(&player, "Previous");
        }
        false
    }

    /// Send a command to the player
    fn send_command(&self, player_name: &str, method: &str) -> bool {
        let proxy = match zbus::blocking::Proxy::new(
            &self.connection,
            player_name,
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        ) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let result: Result<(), zbus::Error> = proxy.call(method, &());
        match result {
            Ok(()) => {
                debug!("Sent {} to {}", method, player_name);
                true
            }
            Err(e) => {
                warn!("Failed to call {}: {}", method, e);
                false
            }
        }
    }
}
