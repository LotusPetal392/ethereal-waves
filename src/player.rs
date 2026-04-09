// SPDX-License-Identifier: GPL-3.0

use crate::helpers::clamp;
use gst::prelude::*;
use gstreamer::{self as gst};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct Player {
    pub playbin: gst::Element,
    queued_uri: Arc<Mutex<Option<String>>>,
    about_to_finish_rx: mpsc::Receiver<()>,
}

impl Player {
    pub fn new() -> Self {
        match gst::init() {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to initialize GStreamer: {:?}", error)
            }
        }

        let playbin = gst::ElementFactory::make("playbin")
            .build()
            .expect("Failed to create playbin.");

        let queued_uri: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let (about_to_finish_tx, about_to_finish_rx) = mpsc::sync_channel::<()>(8);

        // Connect the about-to-finish signal for gapless playback.
        let queued_uri_clone = queued_uri.clone();
        playbin.connect("about-to-finish", false, move |args| {
            let playbin_elem = args[0]
                .get::<gst::Element>()
                .expect("about-to-finish: invalid element arg");

            // If a next URI has been queued, set it now for seamless transition.
            if let Ok(guard) = queued_uri_clone.lock() {
                if let Some(ref uri) = *guard {
                    playbin_elem.set_property("uri", uri);
                    // Notify the main thread that a gapless transition was queued.
                    let _ = about_to_finish_tx.try_send(());
                }
            }

            None
        });

        Self {
            playbin,
            queued_uri,
            about_to_finish_rx,
        }
    }

    pub fn load(&self, uri: &str) {
        self.playbin.set_property("uri", &uri);
    }

    pub fn play(&mut self) {
        match self.playbin.set_state(gst::State::Playing) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to play: {:?}", error);
            }
        }
    }

    pub fn pause(&mut self) {
        match self.playbin.set_state(gst::State::Paused) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to pause: {:?}", error);
            }
        }
    }

    pub fn stop(&mut self) {
        match self.playbin.set_state(gst::State::Null) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to stop: {:?}", error);
            }
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.playbin.set_property("volume", clamp(volume, 0.0, 1.0));
    }

    /// Set (or clear) the URI to be played gaplessly after the current track.
    pub fn set_queued_uri(&self, uri: Option<String>) {
        if let Ok(mut guard) = self.queued_uri.lock() {
            *guard = uri;
        }
    }

    /// Returns `true` if the about-to-finish callback fired since the last call,
    /// meaning a gapless transition was queued. Drains all pending notifications.
    pub fn take_about_to_finish(&self) -> bool {
        let mut fired = false;

        while self.about_to_finish_rx.try_recv().is_ok() {
            fired = true;
        }

        fired
    }
}
