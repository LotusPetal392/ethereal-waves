// SPDX-License-Identifier: GPL-3.0

use crate::playback_state::PlaybackStatus;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use zbus::interface;

pub struct MediaPlayer2Player {
    pub tx: UnboundedSender<MprisCommand>,
    pub playback_status: Arc<Mutex<PlaybackStatus>>,
}

impl MediaPlayer2Player {
    pub fn new(
        tx: UnboundedSender<MprisCommand>,
        playback_status: Arc<Mutex<PlaybackStatus>>,
    ) -> Self {
        Self {
            tx,
            playback_status,
        }
    }
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
impl MediaPlayer2Player {
    fn play(&self) {
        let _ = self.tx.send(MprisCommand::Play);
    }

    fn pause(&self) {
        let _ = self.tx.send(MprisCommand::Pause);
    }

    fn play_pause(&self) {
        let _ = self.tx.send(MprisCommand::PlayPause);
    }

    fn next(&self) {
        let _ = self.tx.send(MprisCommand::Next);
    }

    fn previous(&self) {
        let _ = self.tx.send(MprisCommand::Previous);
    }

    fn stop(&self) {
        let _ = self.tx.send(MprisCommand::Stop);
    }

    fn seek(&self, offset: i64) {
        let _ = self.tx.send(MprisCommand::Seek(offset));
    }

    // Required properties
    #[zbus(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn playback_status(&self) -> &str {
        self.playback_status.lock().unwrap().as_str()
    }
}

pub struct MediaPlayer2;

#[interface(name = "org.mpris.MediaPlayer2")]
impl MediaPlayer2 {
    fn raise(&self) {}

    fn quit(&self) {}

    #[zbus(property)]
    fn can_quit(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn identity(&self) -> &str {
        "Ethereal Waves"
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<&str> {
        vec!["file"]
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<&str> {
        vec![
            "audio/mpeg",
            "audio/ogg",
            "audio/flac",
            "audio/opus",
            "audio/wav",
        ]
    }
}

#[derive(Debug, Clone)]
pub enum MprisCommand {
    Play,
    Pause,
    PlayPause,
    Next,
    Previous,
    Stop,
    Seek(i64),
}
