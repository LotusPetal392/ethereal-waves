use crate::app::{PlaylistKind, SortBy, SortDirection};
use crate::fl;
use crate::library::MediaMetaData;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    id: u32,
    name: String,
    kind: PlaylistKind,
    tracks: Vec<(PathBuf, MediaMetaData)>,
}

impl Playlist {
    pub fn new(name: String) -> Playlist {
        let mut id: u32;
        loop {
            id = rand::rng().random();
            if id != 0 {
                break;
            }
        }
        Self {
            id: id,
            name: name,
            kind: PlaylistKind::User,
            tracks: Vec::new(),
        }
    }

    pub fn library() -> Self {
        Self {
            id: u32::MAX,
            name: fl!("library"),
            kind: PlaylistKind::Library,
            tracks: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    pub fn is_library(&self) -> bool {
        matches!(self.kind, PlaylistKind::Library)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn tracks(&self) -> &[(PathBuf, MediaMetaData)] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn sort(&mut self, sort_by: SortBy, sort_direction: SortDirection) {
        match sort_by {
            SortBy::Artist => {
                self.tracks.sort_by(|a, b| {
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
                self.tracks.sort_by(|a, b| {
                    let ordering = a.1.album.cmp(&b.1.album);
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
            SortBy::Title => {
                self.tracks.sort_by(|a, b| {
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
        self.tracks.push(media);
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
