use std::path::Path;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation};
use parking_lot::RwLock;
use tracing::debug;

use crate::mpris::{MediaInfo, MprisPlayer, PlaybackStatus};

/// Media player widget showing current track and controls
pub struct MediaWidget {
    container: GtkBox,
    album_art: Image,
    title_label: Label,
    artist_label: Label,
    #[allow(dead_code)]
    prev_btn: Button,
    play_pause_btn: Button,
    #[allow(dead_code)]
    next_btn: Button,
    player: Arc<RwLock<Option<MprisPlayer>>>,
}

impl Default for MediaWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaWidget {
    /// Create a new media widget
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 12);
        container.add_css_class("media-widget");
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        // Album art
        let album_art = Image::new();
        album_art.set_pixel_size(64);
        album_art.add_css_class("album-art");
        album_art.set_icon_name(Some("audio-x-generic-symbolic"));
        container.append(&album_art);

        // Track info
        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);
        info_box.set_valign(Align::Center);

        let title_label = Label::new(Some("No media playing"));
        title_label.add_css_class("media-title");
        title_label.set_halign(Align::Start);
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_label.set_max_width_chars(25);
        info_box.append(&title_label);

        let artist_label = Label::new(Some(""));
        artist_label.add_css_class("media-artist");
        artist_label.set_halign(Align::Start);
        artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        artist_label.set_max_width_chars(30);
        info_box.append(&artist_label);

        container.append(&info_box);

        // Controls
        let controls_box = GtkBox::new(Orientation::Horizontal, 4);
        controls_box.set_valign(Align::Center);
        controls_box.add_css_class("media-controls");

        let prev_btn = Button::new();
        prev_btn.set_icon_name("media-skip-backward-symbolic");
        prev_btn.add_css_class("media-button");
        prev_btn.set_tooltip_text(Some("Previous"));
        controls_box.append(&prev_btn);

        let play_pause_btn = Button::new();
        play_pause_btn.set_icon_name("media-playback-start-symbolic");
        play_pause_btn.add_css_class("media-button");
        play_pause_btn.add_css_class("play-pause");
        play_pause_btn.set_tooltip_text(Some("Play/Pause"));
        controls_box.append(&play_pause_btn);

        let next_btn = Button::new();
        next_btn.set_icon_name("media-skip-forward-symbolic");
        next_btn.add_css_class("media-button");
        next_btn.set_tooltip_text(Some("Next"));
        controls_box.append(&next_btn);

        container.append(&controls_box);

        // Initialize MPRIS player
        let mpris_player: Option<MprisPlayer> = MprisPlayer::new().ok();
        let player = Arc::new(RwLock::new(mpris_player));

        // Connect button handlers
        let player_prev = player.clone();
        prev_btn.connect_clicked(move |_| {
            let guard = player_prev.read();
            if let Some(p) = guard.as_ref() {
                p.previous();
            }
        });

        let player_play = player.clone();
        let play_btn_ref = play_pause_btn.clone();
        play_pause_btn.connect_clicked(move |_| {
            let guard = player_play.read();
            if let Some(p) = guard.as_ref() {
                p.play_pause();
                // Toggle icon (will be properly updated on refresh)
                let current_icon = play_btn_ref.icon_name().unwrap_or_default();
                if current_icon == "media-playback-start-symbolic" {
                    play_btn_ref.set_icon_name("media-playback-pause-symbolic");
                } else {
                    play_btn_ref.set_icon_name("media-playback-start-symbolic");
                }
            }
        });

        let player_next = player.clone();
        next_btn.connect_clicked(move |_| {
            let guard = player_next.read();
            if let Some(p) = guard.as_ref() {
                p.next();
            }
        });

        Self {
            container,
            album_art,
            title_label,
            artist_label,
            prev_btn,
            play_pause_btn,
            next_btn,
            player,
        }
    }

    /// Refresh media info
    pub fn refresh(&self) {
        debug!("Refreshing media widget");
        let player_guard = self.player.read();

        let player_ref: Option<&MprisPlayer> = player_guard.as_ref();
        if let Some(player) = player_ref {
            let players = player.find_players();
            debug!("Available players: {:?}", players);

            let media_info: Option<MediaInfo> = player.get_current_media();
            if let Some(info) = media_info {
                debug!(
                    "Got media info: title={}, artist={}",
                    info.title, info.artist
                );
                // Update title
                if !info.title.is_empty() {
                    self.title_label.set_text(&info.title);
                } else {
                    self.title_label.set_text("Unknown Title");
                }

                // Update artist
                if !info.artist.is_empty() {
                    self.artist_label.set_text(&info.artist);
                    self.artist_label.set_visible(true);
                } else if !info.album.is_empty() {
                    self.artist_label.set_text(&info.album);
                    self.artist_label.set_visible(true);
                } else {
                    self.artist_label.set_visible(false);
                }

                // Update play/pause button icon
                match info.status {
                    Some(PlaybackStatus::Playing) => {
                        self.play_pause_btn
                            .set_icon_name("media-playback-pause-symbolic");
                    }
                    _ => {
                        self.play_pause_btn
                            .set_icon_name("media-playback-start-symbolic");
                    }
                }

                // Update album art
                if let Some(art_url) = &info.art_url {
                    self.load_album_art(art_url);
                } else {
                    self.album_art
                        .set_icon_name(Some("audio-x-generic-symbolic"));
                }

                self.container.set_visible(true);
                debug!("Media widget updated: {} - {}", info.title, info.artist);
            } else {
                self.show_no_media();
            }
        } else {
            self.show_no_media();
        }
    }

    /// Show no media message
    fn show_no_media(&self) {
        self.title_label.set_text("No media playing");
        self.artist_label.set_text("");
        self.artist_label.set_visible(false);
        self.album_art
            .set_icon_name(Some("audio-x-generic-symbolic"));
        self.play_pause_btn
            .set_icon_name("media-playback-start-symbolic");
    }

    /// Load album art from URL
    fn load_album_art(&self, url: &str) {
        // Handle file:// URLs
        if url.starts_with("file://") {
            let path = url.strip_prefix("file://").unwrap_or(url);
            if Path::new(path).exists() {
                self.album_art.set_from_file(Some(path));
                return;
            }
        }

        // Handle http/https URLs - load asynchronously
        if url.starts_with("http://") || url.starts_with("https://") {
            // For now, just use a generic icon for remote URLs
            // Full implementation would download and cache the image
            self.album_art
                .set_icon_name(Some("audio-x-generic-symbolic"));
            return;
        }

        // Try as local path
        if Path::new(url).exists() {
            self.album_art.set_from_file(Some(url));
        } else {
            self.album_art
                .set_icon_name(Some("audio-x-generic-symbolic"));
        }
    }

    /// Get the widget container
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Check if any media player is available
    pub fn has_player(&self) -> bool {
        let guard = self.player.read();
        if let Some(player) = guard.as_ref() {
            !player.find_players().is_empty()
        } else {
            false
        }
    }
}
