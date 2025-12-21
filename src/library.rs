use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use xdg::BaseDirectories;

#[derive(Debug, Clone)]
pub struct Library {
    pub media: HashMap<PathBuf, MediaMetaData>,
}

impl Library {
    pub fn new() -> Library {
        Self {
            media: HashMap::new(),
        }
    }

    // Save the current media to the xdg data directory
    pub fn save(&self, xdg_dirs: BaseDirectories) -> Result<(), Box<dyn Error>> {
        let file_path = xdg_dirs.place_data_file("library.json").unwrap();
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.media)?;
        writer.flush()?;
        Ok(())
    }

    // Load media from the xdg data directory if it exists
    pub fn load(
        &self,
        xdg_dirs: BaseDirectories,
    ) -> Result<HashMap<PathBuf, MediaMetaData>, Box<dyn Error>> {
        let file_path = xdg_dirs.get_data_file("library.json").unwrap();
        let data = fs::read_to_string(file_path)?;
        let json: HashMap<PathBuf, MediaMetaData> = serde_json::from_str(&data)?;

        Ok(json)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaMetaData {
    pub id: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub album_disc_number: Option<u32>,
    pub album_disc_count: Option<u32>,
    pub track_number: Option<u32>,
    pub track_count: Option<u32>,
    pub duration: Option<u64>,
    pub encoder: Option<String>,
    pub comment: Option<String>,
    pub extended_comment: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate: Option<u32>,
    pub container_format: Option<String>,
}

impl MediaMetaData {
    pub fn new() -> Self {
        Self {
            id: None,
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            genre: None,
            album_disc_number: None,
            album_disc_count: None,
            track_number: None,
            track_count: None,
            duration: None,
            encoder: None,
            comment: None,
            extended_comment: None,
            audio_codec: None,
            bitrate: None,
            container_format: None,
        }
    }
}
