// SPDX-License-Identifier: GPL-3.0

use crate::app::{PlaylistKind, SortBy, SortDirection};
use crate::fl;
use crate::library::MediaMetaData;
use chrono::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    id: u32,
    name: String,
    kind: PlaylistKind,
    tracks: Vec<Track>,
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

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn sort(&mut self, sort_by: SortBy, sort_direction: SortDirection) {
        match sort_by {
            SortBy::Artist => {
                self.tracks.sort_by(|a, b| {
                    let ordering = a
                        .metadata
                        .artist
                        .cmp(&b.metadata.artist)
                        .then(a.metadata.album.cmp(&b.metadata.album))
                        .then(a.metadata.track_number.cmp(&b.metadata.track_number));
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
            SortBy::Album => {
                self.tracks.sort_by(|a, b| {
                    let ordering = a.metadata.album.cmp(&b.metadata.album);
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
            SortBy::Title => {
                self.tracks.sort_by(|a, b| {
                    let ordering = a.metadata.title.cmp(&b.metadata.title);
                    match sort_direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }
        }
    }

    pub fn push(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn select(&mut self, index: usize) {
        self.tracks[index].selected = true;
    }

    pub fn selected(&self) -> Vec<&Track> {
        self.tracks.iter().filter(|t| t.selected).collect()
    }

    pub fn deselect(&mut self, index: usize) {
        self.tracks[index].selected = false;
    }

    pub fn clear_selected(&mut self) {
        self.tracks.iter_mut().for_each(|t| t.selected = false);
    }

    pub fn remove_selected(&mut self) {
        self.tracks.retain(|t| !t.selected);
    }

    pub fn selected_iter(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter().filter(|t| t.selected)
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        if start < end {
            for i in start..=end {
                self.tracks[i].selected = true;
            }
        } else if end < start {
            for i in end..=start {
                self.tracks[i].selected = true;
            }
        }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Track {
    pub path: PathBuf,
    #[serde(skip)]
    pub selected: bool,
    pub metadata: MediaMetaData,
    pub date_added: String,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            selected: false,
            metadata: MediaMetaData::new(),
            date_added: Local::now().to_string(),
        }
    }
}

impl Track {
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            selected: false,
            metadata: MediaMetaData::new(),
            date_added: Local::now().to_string(),
        }
    }
}
