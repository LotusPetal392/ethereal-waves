//use crate::app::Message;
//use cosmic::iced::Subscription;
//use cosmic::iced::futures::{self, SinkExt, channel::mpsc::Sender};
use gst::prelude::*;
use gstreamer as gst;
//use std::sync::mpsc::Receiver;

pub struct Player {
    pub playbin: gst::Element,
    pub pipeline: gst::Pipeline,
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

        Self { playbin, pipeline }
    }

    pub fn load(&self, uri: &str) {
        self.playbin.set_property("uri", &uri);
    }

    pub fn play(&self) {
        let _ = self.pipeline.set_state(gst::State::Playing);
    }

    pub fn pause(&self) {
        let _ = self.pipeline.set_state(gst::State::Paused);
    }

    pub fn stop(&self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
