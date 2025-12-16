// SPDX-License-Identifier: GPL-3.0

mod app;
mod config;
mod footer;
mod i18n;
mod key_bind;
mod library;
mod menu;

use app::Flags;
use config::Config;
use cosmic::{app::Settings, iced::Limits};

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    let (config_handler, config) = Config::load();

    // Settings for configuring the application window and iced runtime.
    let mut settings: Settings = Settings::default();
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.theme(config.app_theme.theme());

    let flags = Flags { config_handler };

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, flags)
}
