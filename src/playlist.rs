use crate::app::{SortBy, SortDirection};
use crate::library::MediaMetaData;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    id: u32,
    name: String,
    media: Vec<(PathBuf, MediaMetaData)>,
}

impl Playlist {
    pub fn new(name: String) -> Playlist {
        let id = match name.len() {
            0 => 0,
            _ => rand::rng().random(),
        };
        Self {
            id: id,
            name: name,
            media: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn media(&self) -> Vec<(PathBuf, MediaMetaData)> {
        self.media.clone()
    }

    pub fn len(&self) -> usize {
        self.media.len()
    }

    pub fn sort(&mut self, sort_by: SortBy, sort_direction: SortDirection) {
        match sort_by {
            SortBy::Artist => {
                self.media.sort_by(|a, b| {
                    let ordering =
                        a.1.artist
                            .cmp(&b.1.artist)
                            .then(a.1.album.cmp(&b.1.album))
                            .then(a.1.track_number.cmp(&b.1.track_number));
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
            SortBy::Album => {
                self.media.sort_by(|a, b| {
                    let ordering = a.1.album.cmp(&b.1.album);
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
            SortBy::Title => {
                self.media.sort_by(|a, b| {
                    let ordering = a.1.title.cmp(&b.1.title);
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
        }
    }

    pub fn push(&mut self, media: (PathBuf, MediaMetaData)) {
        self.media.push(media);
    }

    pub fn remove(&mut self, index: usize) {
        self.media.remove(index);
    }
}

impl fmt::Debug for Playlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Playlist {{ id: {}, name: {}, tracks: {:?} }}",
            self.id, self.name, self.media
        )
    }
}
