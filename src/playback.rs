use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Error, anyhow};
#[cfg(feature = "with-player")]
use proteus_lib::playback::player::Player;
#[cfg(feature = "with-player")]
use proteus_lib::tools::decode::check_audio_file_supported;

#[derive(Debug)]
pub enum PlaybackLoadError {
    UnsupportedFormat { file_name: String },
    Other(Error),
}

impl PlaybackLoadError {
    fn other(error: impl Into<Error>) -> Self {
        Self::Other(error.into())
    }
}

impl fmt::Display for PlaybackLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat { file_name } => {
                write!(f, "{file_name} is in an unsupported format")
            }
            Self::Other(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PlaybackLoadError {}

#[derive(Debug, Clone, Copy)]
pub struct PlaybackStatus {
    pub duration: Option<f64>,
    pub time: f64,
    pub volume: f32,
    pub playing: bool,
}

pub struct PlaybackController {
    #[cfg(feature = "with-player")]
    player: Option<Player>,
    #[cfg(not(feature = "with-player"))]
    player: Option<()>,
    current_path: Option<PathBuf>,
}

impl PlaybackController {
    pub fn new() -> Self {
        Self {
            player: None,
            current_path: None,
        }
    }

    pub fn load(&mut self, path: &Path) -> Result<(), PlaybackLoadError> {
        preflight_supported_format(path)?;

        // Drop any existing player before replacing it.
        self.shutdown();

        #[cfg(feature = "with-player")]
        {
            let path_string = path
                .to_str()
                .ok_or_else(|| PlaybackLoadError::other(anyhow!("path contains invalid UTF-8")))?
                .to_owned();

            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_ascii_lowercase);
            let player = match extension.as_deref() {
                Some("prot") | Some("mka") => Player::new(&path_string),
                _ => Player::new_from_file_paths_legacy(vec![vec![path_string.clone()]]),
            };

            player.set_max_sink_chunks(30);

            self.player = Some(player);
            self.current_path = Some(path.to_path_buf());
            Ok(())
        }

        #[cfg(not(feature = "with-player"))]
        {
            let _ = path
                .to_str()
                .ok_or_else(|| PlaybackLoadError::other(anyhow!("path contains invalid UTF-8")))?;
            self.player = None;
            self.current_path = Some(path.to_path_buf());
            Ok(())
        }
    }

    pub fn status(&self) -> PlaybackStatus {
        #[cfg(feature = "with-player")]
        {
            match &self.player {
                Some(player) => PlaybackStatus {
                    duration: Some(player.get_duration()),
                    time: player.get_time(),
                    volume: player.get_volume(),
                    playing: player.is_playing(),
                },
                None => PlaybackStatus {
                    duration: None,
                    time: 0.0,
                    volume: 1.0,
                    playing: false,
                },
            }
        }

        #[cfg(not(feature = "with-player"))]
        {
            PlaybackStatus {
                duration: None,
                time: 0.0,
                volume: 1.0,
                playing: false,
            }
        }
    }

    pub fn play_pause(&mut self) {
        #[cfg(feature = "with-player")]
        {
            let Some(player) = &mut self.player else {
                return;
            };

            if player.is_playing() {
                player.pause();
            } else {
                player.play();
            }
        }
    }

    pub fn stop(&mut self) {
        #[cfg(feature = "with-player")]
        {
            let Some(player) = &mut self.player else {
                return;
            };

            player.stop();
            player.refresh_tracks();
        }
    }

    pub fn shutdown(&mut self) {
        #[cfg(feature = "with-player")]
        if let Some(player) = &self.player {
            player.stop();
        }

        self.player = None;
        self.current_path = None;
    }

    pub fn reset(&mut self) {
        self.stop();
    }

    pub fn shuffle(&mut self) {
        #[cfg(feature = "with-player")]
        {
            if let Some(player) = &mut self.player {
                player.refresh_tracks();
            }
        }
    }

    pub fn seek(&mut self, position_seconds: f64) {
        #[cfg(not(feature = "with-player"))]
        let _ = position_seconds;

        #[cfg(feature = "with-player")]
        {
            if let Some(player) = &mut self.player {
                player.seek(position_seconds.max(0.0));
            }
        }
    }

    pub fn seek_by(&mut self, offset_seconds: f64) {
        let status = self.status();
        let duration = status.duration.unwrap_or(f64::INFINITY);
        let next = (status.time + offset_seconds).clamp(0.0, duration);
        self.seek(next);
    }

    pub fn set_volume(&mut self, volume: f32) {
        #[cfg(not(feature = "with-player"))]
        let _ = volume;

        #[cfg(feature = "with-player")]
        {
            if let Some(player) = &mut self.player {
                player.set_volume(volume.clamp(0.0, 1.0));
            }
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.player.is_some()
    }
}

fn preflight_supported_format(path: &Path) -> Result<(), PlaybackLoadError> {
    #[cfg(not(feature = "with-player"))]
    {
        let _ = path;
        return Ok(());
    }

    #[cfg(feature = "with-player")]
    {
        let path_string = path
            .to_str()
            .ok_or_else(|| PlaybackLoadError::other(anyhow!("path contains invalid UTF-8")))?;

        if check_audio_file_supported(path_string).supported {
            Ok(())
        } else {
            Err(PlaybackLoadError::UnsupportedFormat {
                file_name: display_file_name(path),
            })
        }
    }
}

fn display_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

impl Drop for PlaybackController {
    fn drop(&mut self) {
        self.shutdown();
    }
}
