// SPDX-License-Identifier: GPL-3.0

use gst::State;
use gst::prelude::*;
use gstreamer::{self as gst};

pub struct Player {
    pub playbin: gst::Element,
    pub pipeline: gst::Pipeline,
    current_state: State,
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

        let pipeline = playbin.clone().dynamic_cast::<gst::Pipeline>().unwrap();

        let current_state = State::Null;

        Self {
            playbin,
            pipeline,
            current_state,
        }
    }

    pub fn load(&self, uri: &str) {
        self.playbin.set_property("uri", &uri);
    }

    pub fn play(&self) {
        match self.pipeline.set_state(gst::State::Playing) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to play: {:?}", error);
            }
        }
    }

    pub fn pause(&self) {
        match self.pipeline.set_state(gst::State::Paused) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to pause: {:?}", error);
            }
        }
    }

    pub fn stop(&self) {
        match self.pipeline.set_state(gst::State::Null) {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to stop: {:?}", error);
            }
        }
    }

    pub fn get_current_state(&self) -> &State {
        &self.current_state
    }

    pub fn set_current_state(&mut self, current_state: State) {
        self.current_state = current_state;
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.playbin.set_property("volume", volume);
    }
}
