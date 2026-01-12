use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error};

pub struct SoundPlayer {
    _stream: OutputStream,
    sink: Sink,
}

impl SoundPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            _stream: stream,
            sink,
        })
    }

    pub fn play_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        debug!("Playing sound file: {}", path.display());

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;
        self.sink.append(source);

        Ok(())
    }

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

        for base_path in &search_paths {
            // Try freedesktop sound theme
            for ext in &extensions {
                let path = format!("{}/freedesktop/stereo/{}.{}", base_path, name, ext);
                if Path::new(&path).exists() {
                    return self.play_file(&path);
                }
            }

            // Try direct path
            for ext in &extensions {
                let path = format!("{}/{}.{}", base_path, name, ext);
                if Path::new(&path).exists() {
                    return self.play_file(&path);
                }
            }
        }

        error!("Sound '{}' not found in any search path", name);
        Ok(())
    }

    pub fn stop(&self) {
        self.sink.stop();
    }
}

impl Default for SoundPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize sound player")
    }
}
