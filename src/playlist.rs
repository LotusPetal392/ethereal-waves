use crate::library::MediaMetaData;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, path::PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    id: u32,
    name: String,
    pub tracks: Vec<(usize, String)>,
}

impl Playlist {
    pub fn new() -> Playlist {
        Self {
            id: 0,
            name: String::new(),
            tracks: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn push(&mut self, id: String) {
        self.tracks.push((self.tracks.len(), id));
    }

    pub fn remove(&mut self, index: usize) {
        self.tracks.remove(index);
    }
}

impl fmt::Debug for Playlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Playlist {{ id: {}, name: {}, tracks: {:?} }}",
            self.id, self.name, self.tracks
        )
    }
}
