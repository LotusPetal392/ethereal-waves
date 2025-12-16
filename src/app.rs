// SPDX-License-Identifier: GPL-3.0

use crate::config::{AppTheme, CONFIG_VERSION, Config};
use crate::fl;
use crate::footer::footer;
use crate::key_bind::key_binds;
use crate::library::{Library, MediaMetaData};
use crate::menu::menu_bar;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::theme;
use cosmic::widget::{
    self,
    about::About,
    icon,
    menu::{self, Action},
    nav_bar,
};
use cosmic::{
    cosmic_theme,
    dialog::file_chooser,
    iced::{
        Alignment, Length, Subscription,
        alignment::{Horizontal, Vertical},
        event::{self, Event},
        keyboard::{Event as KeyEvent, Key, Modifiers},
    },
};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use gstreamer as gst;
use gstreamer_pbutils as pbutils;
use std::{collections::HashMap, process, sync::Arc, time::Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;
use urlencoding::decode;
use walkdir::WalkDir;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Time active
    time: u32,
    /// Toggle the watch subscription
    watch_is_active: bool,
    /// Settings page / app theme dropdown labels
    app_theme_labels: Vec<String>,

    config_handler: Option<cosmic_config::Config>,

    library: Library,
    is_updating: bool,
    playback_progress: f32,
    update_progress: f32,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    AddLibraryDialog,
    AppTheme(AppTheme),
    Cancelled,
    Key(Modifiers, Key),
    LaunchUrl(String),
    LibraryPathOpenError(Arc<file_chooser::Error>),
    PlaybackTimeChanged(f32),
    Quit,
    RemoveLibraryPath(String),
    SelectedPaths(Vec<String>),
    ToggleContextPage(ContextPage),
    ToggleWatch,
    TransportPrevious,
    TransportPlay,
    TransportNext,
    UpdateComplete(Library),
    UpdateConfig(Config),
    UpdateLibrary,
    UpdateProgress(f32),
    WatchTick(u32),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = Flags;

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.LotusPetal392.ethereal-waves";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("page-id", num = 1))
            .data::<Page>(Page::Page1)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("page-id", num = 2))
            .data::<Page>(Page::Page2)
            .icon(icon::from_name("applications-system-symbolic"));

        nav.insert()
            .text(fl!("page-id", num = 3))
            .data::<Page>(Page::Page3)
            .icon(icon::from_name("applications-games-symbolic"));

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: key_binds(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, CONFIG_VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            time: 0,
            watch_is_active: false,
            app_theme_labels: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            config_handler: _flags.config_handler,
            library: Library::new(),
            is_updating: false,
            playback_progress: 0.0,
            update_progress: 0.0,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu_bar(self.is_updating, &self.key_binds);
        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Page1 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 1)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let counter_label = ["Watch: ", self.time.to_string().as_str()].concat();
                let section = cosmic::widget::settings::section().add(
                    cosmic::widget::settings::item::builder(counter_label).control(
                        widget::button::text(if self.watch_is_active {
                            "Stop"
                        } else {
                            "Start"
                        })
                        .on_press(Message::ToggleWatch),
                    ),
                );

                widget::column::with_capacity(2)
                    .push(header)
                    .push(section)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page2 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 2)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page3 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 3)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }
        };

        widget::container(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::Key(modifiers, key))
                }
                _ => None,
            }),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Conditionally enables a timer that emits a message every second.
        if self.watch_is_active {
            subscriptions.push(Subscription::run(|| {
                iced_futures::stream::channel(1, |mut emitter| async move {
                    let mut time = 1;
                    let mut interval = tokio::time::interval(Duration::from_secs(1));

                    loop {
                        interval.tick().await;
                        _ = emitter.send(Message::WatchTick(time)).await;
                        time += 1;
                    }
                })
            }));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> cosmic::Task<cosmic::Action<Self::Message>> {
        // Helper for updating configuration
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](&config_handler, $value) }
                        {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        match message {
            Message::AddLibraryDialog => {
                return cosmic::task::future(async move {
                    let dialog = file_chooser::open::Dialog::new().title(fl!("add-new-location"));

                    match dialog.open_folders().await {
                        Ok(response) => {
                            let mut paths: Vec<String> = Vec::new();

                            for u in response.urls() {
                                if let Ok(decoded) = decode(u.path()) {
                                    paths.push(decoded.into_owned());
                                } else {
                                    println!("Can't decode URL.");
                                }
                            }
                            Message::SelectedPaths(paths)
                        }
                        Err(file_chooser::Error::Cancelled) => Message::Cancelled,
                        Err(why) => Message::LibraryPathOpenError(Arc::new(why)),
                    }
                });
            }

            Message::PlaybackTimeChanged(time) => {
                self.playback_progress = time;
                println!("playback time changed: {}", time);
            }

            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }

            Message::Cancelled => {}

            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }

            Message::LibraryPathOpenError(why) => {
                log::error!("{why}");
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::Quit => {
                process::exit(0);
            }

            Message::RemoveLibraryPath(path) => {
                let mut library_paths = self.config.library_paths.clone();
                library_paths.remove(&path);
                config_set!(library_paths, library_paths);
            }

            Message::SelectedPaths(paths) => {
                let mut library_paths = self.config.library_paths.clone();

                for path in paths {
                    library_paths.insert(path);
                }

                config_set!(library_paths, library_paths);
            }

            Message::ToggleWatch => {
                self.watch_is_active = !self.watch_is_active;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::TransportPrevious => {
                println!("Previous")
            }

            Message::TransportPlay => {
                println!("Play/Pause")
            }

            Message::TransportNext => {
                println!("Next")
            }

            Message::UpdateComplete(library) => {
                self.library = library;
                self.is_updating = false;
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::UpdateLibrary => {
                // TODO: Make this suck less and add error handling
                if self.is_updating {
                    return Task::none();
                }
                self.is_updating = true;
                self.update_progress = 0.0;

                let library_paths = self.config.library_paths.clone();

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                std::thread::spawn(move || {
                    let mut library: Library = Library::new();
                    let valid_extensions = [
                        "flac".to_string(),
                        "m4a".to_string(),
                        "mp3".to_string(),
                        "ogg".to_string(),
                        "opus".to_string(),
                    ];

                    // Get paths
                    for path in library_paths {
                        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                            let extension = entry
                                .file_name()
                                .to_str()
                                .unwrap_or("")
                                .split(".")
                                .last()
                                .unwrap_or("");
                            let size = entry.metadata().unwrap().len();

                            if valid_extensions.contains(&extension.to_string())
                                && size > 4096 as u64
                            {
                                library
                                    .media
                                    .insert(entry.into_path(), MediaMetaData::new());
                            }
                        }
                    }

                    // Get metadata
                    gst::init().unwrap();

                    let discoverer = match pbutils::Discoverer::new(gst::ClockTime::from_seconds(5))
                    {
                        Ok(discoverer) => discoverer,
                        Err(error) => panic!("Failed to create discoverer: {:?}", error),
                    };

                    let mut update_progress: f32 = 0.0;
                    let mut update_percent_old: f32 = 0.0;
                    let update_total: f32 = library.media.len() as f32;

                    library.media.iter_mut().for_each(|(file, track_metadata)| {
                        let file_str = match file.to_str() {
                            Some(file_str) => file_str,
                            None => "",
                        };

                        let uri = Url::from_file_path(file_str).unwrap();

                        let info = discoverer
                            .discover_uri(&uri.as_str())
                            .expect("Cannot read file.");

                        // Read tags
                        if let Some(tags) = info.tags() {
                            // Title
                            track_metadata.title =
                                tags.get::<gst::tags::Title>().map(|t| t.get().to_owned());
                            // Artist
                            track_metadata.artist =
                                tags.get::<gst::tags::Artist>().map(|t| t.get().to_owned());
                            // Album
                            track_metadata.album =
                                tags.get::<gst::tags::Album>().map(|t| t.get().to_owned());
                            //Album Artist
                            track_metadata.album_artist = tags
                                .get::<gst::tags::AlbumArtist>()
                                .map(|t| t.get().to_owned());
                            // Genre
                            track_metadata.genre =
                                tags.get::<gst::tags::Genre>().map(|t| t.get().to_owned());
                            // Track Number
                            track_metadata.track_number = tags
                                .get::<gst::tags::TrackNumber>()
                                .map(|t| t.get().to_owned());
                            // Track Count
                            track_metadata.track_count = tags
                                .get::<gst::tags::TrackCount>()
                                .map(|t| t.get().to_owned());
                            // Disc Number
                            track_metadata.album_disc_number = tags
                                .get::<gst::tags::AlbumVolumeNumber>()
                                .map(|t| t.get().to_owned());
                            // Disc Count
                            track_metadata.album_disc_count = tags
                                .get::<gst::tags::AlbumVolumeCount>()
                                .map(|t| t.get().to_owned());
                            // Duration
                            if let Some(duration) = info.duration() {
                                track_metadata.duration = Some(duration.seconds());
                            }
                        } else {
                            // If there's no metadata just fill in the filename
                            track_metadata.title = Some(file.to_string_lossy().to_string());
                        }

                        // Update progress bar
                        update_progress = update_progress + 1.0;
                        if update_percent_old != (update_progress / update_total * 100.0).round() {
                            _ = tx.send(Message::UpdateProgress(
                                (update_progress / update_total * 100.0).round(),
                            ));
                        }
                        update_percent_old = (update_progress / update_total * 100.0).round();
                    });

                    std::thread::sleep(tokio::time::Duration::from_secs(1));
                    _ = tx.send(Message::UpdateComplete(library));
                });

                return cosmic::Task::stream(UnboundedReceiverStream::new(rx))
                    .map(cosmic::Action::App);
            }

            Message::WatchTick(time) => {
                self.time = time;
            }

            Message::UpdateProgress(progress) => {
                self.update_progress = progress;
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }

    /// Footer area
    fn footer(&self) -> Option<Element<'_, Message>> {
        Some(
            footer(
                self.is_updating,
                self.playback_progress,
                self.update_progress,
            )
            .into(),
        )
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn settings(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };

        let mut library_column = widget::column();
        library_column = library_column.push(
            widget::button::text(fl!("add-new-location")).on_press(Message::AddLibraryDialog),
        );

        let library_paths_length = self.config.library_paths.len() - 1;

        for (i, path) in self.config.library_paths.iter().enumerate() {
            let mut path_row = widget::row::with_capacity(2);
            // Adds text
            path_row =
                path_row.push(widget::text::text(path.clone()).width(Length::FillPortion(1)));
            // Adds delete button
            path_row = path_row.push(
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .on_press(Message::RemoveLibraryPath(path.clone())),
            );
            library_column = library_column.push(path_row.width(Length::Fill).padding(space_xxs));

            if i < library_paths_length {
                library_column = library_column.push(widget::divider::horizontal::light());
            }
        }

        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
                .add({
                    widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                        &self.app_theme_labels,
                        Some(app_theme_selected),
                        move |index| {
                            Message::AppTheme(match index {
                                1 => AppTheme::Dark,
                                2 => AppTheme::Light,
                                _ => AppTheme::System,
                            })
                        },
                    ))
                })
                .into(),
            widget::settings::section()
                .title(fl!("library"))
                .add(library_column)
                .into(),
        ])
        .into()
    }

    fn update_config(&mut self) -> Task<cosmic::Action<Message>> {
        cosmic::command::set_theme(self.config.app_theme.theme())
    }
}

/// Flags passed into the app
#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
}

/// The page to display in the application.
pub enum Page {
    Page1,
    Page2,
    Page3,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    Quit,
    UpdateLibrary,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::Quit => Message::Quit,
            MenuAction::UpdateLibrary => Message::UpdateLibrary,
        }
    }
}
