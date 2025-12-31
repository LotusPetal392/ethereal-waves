// SPDX-License-Identifier: GPL-3.0

use crate::app::AppModel;
use cosmic::{
    Application,
    cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    iced::Subscription,
    theme,
};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, collections::HashSet};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[version = 1]
#[serde(default)]
pub struct Config {
    pub app_theme: AppTheme,
    pub library_paths: HashSet<String>,
}

impl Config {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(AppModel::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Self::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {errs:?}");
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {err}");
                (None, Self::default())
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            library_paths: HashSet::new(),
        }
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct State {
    pub window_height: f32,
    pub window_width: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            window_height: 1024.0,
            window_width: 768.0,
        }
    }
}

impl State {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new_state(AppModel::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Self::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {errs:?}");
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {err}");
                (None, Self::default())
            }
        }
    }

    pub fn subscription() -> Subscription<cosmic_config::Update<Self>> {
        struct ConfigSubscription;
        cosmic_config::config_state_subscription(
            TypeId::of::<ConfigSubscription>(),
            AppModel::APP_ID.into(),
            CONFIG_VERSION,
        )
    }
}
