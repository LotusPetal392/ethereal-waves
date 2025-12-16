use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub media: HashMap<PathBuf, MediaMetaData>,
}

impl Library {
    pub fn new() -> Library {
        Self {
            media: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaMetaData {
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
}

impl MediaMetaData {
    pub fn new() -> Self {
        Self {
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
        }
    }
}
