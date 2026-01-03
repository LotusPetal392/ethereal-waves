use crate::library::MediaMetaData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Playlist {
    name: String,
    tracks: Vec<(usize, String)>,
}

impl Playlist {
    pub fn new() -> Playlist {
        Self {
            name: String::new(),
            tracks: Vec::new(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn rename(&mut self, name: String) {
        self.name = name
    }

    pub fn push(&mut self, id: String) {
        self.tracks.push((self.tracks.len(), id));
    }

    pub fn remove(&mut self, index: usize) {
        self.tracks.remove(index);
    }
}
