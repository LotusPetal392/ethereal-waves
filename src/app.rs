// SPDX-License-Identifier: GPL-3.0

use crate::config::{AppTheme, CONFIG_VERSION, Config, State};
use crate::fl;
use crate::footer::footer;
use crate::key_bind::key_binds;
use crate::library::{Library, MediaMetaData};
use crate::menu::menu_bar;
use crate::page::empty_library;
use crate::page::list_view;
use crate::player::Player;
use cosmic::app::context_drawer;
use cosmic::iced::keyboard::key::Named;
use cosmic::iced_widget::scrollable;
use cosmic::prelude::*;
use cosmic::{
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme,
    dialog::file_chooser,
    iced::{
        self, Alignment, Length, Size, Subscription,
        alignment::{Horizontal, Vertical},
        event::{self, Event},
        keyboard::{Event as KeyEvent, Key, Modifiers},
        window::Event as WindowEvent,
    },
    theme,
    widget::{
        self, Column, Row,
        about::About,
        icon,
        menu::{self, Action},
        nav_bar, toggler,
    },
};
use gst::prelude::ElementExt;
use gst::prelude::ElementExtManual;
use gstreamer as gst;
use gstreamer_pbutils as pbutils;
use sha256::digest;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;
use urlencoding::decode;
use walkdir::WalkDir;
use xdg::BaseDirectories;

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
    pub config: Config,
    /// Settings page / app theme dropdown labels
    app_theme_labels: Vec<String>,

    config_handler: Option<cosmic_config::Config>,
    state_handler: Option<cosmic_config::Config>,
    state: crate::config::State,

    pub xdg_dirs: BaseDirectories,

    pub library: Library,

    pub is_updating: bool,
    pub playback_progress: f32,
    pub update_progress: f32,
    pub update_total: f32,
    pub update_percent: f32,

    dragging_progress_slider: bool,

    player: Player,

    pub now_playing: Option<MediaMetaData>,
    pub artwork_dir: Option<PathBuf>,
    album_artwork: HashMap<String, Vec<u8>>,

    size_multiplier: f32,
    pub list_row_height: f32,
    pub list_view_scroll_offset: f32,
    pub list_start: usize,
    pub list_visible_row_count: usize,
    pub list_selected: Vec<String>,
    list_last_clicked: Option<Instant>,

    control_pressed: u8,
    shift_pressed: u8,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    AddLibraryDialog,
    AppTheme(AppTheme),
    Cancelled,
    ChangeTrack(String),
    KeyPressed(Modifiers, Key),
    KeyReleased(Key),
    LaunchUrl(String),
    LibraryPathOpenError(Arc<file_chooser::Error>),
    ListSelectRow(String),
    ListViewScroll(scrollable::Viewport),
    Next,
    NewPlaylist,
    Previous,
    Quit,
    ReleaseSlider,
    RemoveLibraryPath(String),
    SelectedPaths(Vec<String>),
    SizeDecrease,
    SizeIncrease,
    SliderSeek(f32),
    Tick,
    ToggleContextPage(ContextPage),
    ToggleListTextWrap(bool),
    TogglePlaying,
    UpdateComplete(Library),
    UpdateConfig(Config),
    UpdateLibrary,
    UpdateProgress(f32, f32, f32),
    WindowResized(Size),
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
            .text(fl!("library"))
            .data::<Page>(Page::Library)
            .icon(icon::from_name("folder-music-symbolic"))
            .activate();

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
            config: cosmic_config::Config::new(Self::APP_ID, CONFIG_VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            app_theme_labels: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            config_handler: _flags.config_handler,
            state_handler: _flags.state_handler,
            state: _flags.state.clone(),
            xdg_dirs: xdg::BaseDirectories::with_prefix(Self::APP_ID),
            library: Library::new(),
            is_updating: false,
            playback_progress: 0.0,
            update_progress: 0.0,
            update_total: 0.0,
            update_percent: 0.0,
            dragging_progress_slider: false,
            player: Player::new(),
            now_playing: None,
            artwork_dir: None,
            album_artwork: HashMap::new(),
            size_multiplier: _flags.state.size_multiplier,
            list_row_height: 20.0,
            list_view_scroll_offset: 0.0,
            list_start: 0,
            list_visible_row_count: 0,
            list_selected: Vec::new(),
            list_last_clicked: None,
            control_pressed: 0,
            shift_pressed: 0,
        };

        // Load library
        app.library.media = match app.library.load(app.xdg_dirs.clone()) {
            Ok(library) => library,
            Err(error) => {
                eprintln!("Can't open library: {:?}", error);
                let library: HashMap<PathBuf, MediaMetaData> = HashMap::new();
                library
            }
        };

        // TODO: Load playlists

        // Create a startup command that sets the window title.
        let command = app.update_title();

        // Build out artwork cache directory
        app.artwork_dir = app.xdg_dirs.get_cache_home();
        app.artwork_dir = Some(app.artwork_dir.unwrap().join("artwork"));

        app.update_list_row_height();

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
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Library => {
                if self.library.media.len() == 0 {
                    empty_library::content()
                } else {
                    list_view::content(&self)
                }
            }
        };

        widget::container(widget::column::with_children(vec![content]))
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
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
                    Some(Message::KeyPressed(modifiers, key))
                }
                Event::Keyboard(KeyEvent::KeyReleased { key, .. }) => {
                    Some(Message::KeyReleased(key))
                }
                Event::Window(WindowEvent::CloseRequested) => Some(Message::Quit),
                Event::Window(WindowEvent::Closed) => Some(Message::Quit),
                Event::Window(WindowEvent::Resized(size)) => Some(Message::WindowResized(size)),
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

        if self.now_playing.is_some() {
            subscriptions
                .push(iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick));
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

        // Helper for updating application state
        macro_rules! state_set {
            ($name: ident, $value: expr) => {
                match &self.state_handler {
                    Some(state_handler) => {
                        match paste::paste! { self.state.[<set_ $name>](&state_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!("failed to save state {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.state.$name = $value;
                        log::warn!(
                            "failed to save state {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        match message {
            Message::AddLibraryDialog => {
                return cosmic::task::future(async move {
                    let dialog = file_chooser::open::Dialog::new().title(fl!("add-location"));

                    match dialog.open_folders().await {
                        Ok(response) => {
                            let mut paths: Vec<String> = Vec::new();

                            for u in response.urls() {
                                if let Ok(decoded) = decode(u.path()) {
                                    paths.push(decoded.into_owned());
                                } else {
                                    eprintln!("Can't decode URL.");
                                }
                            }
                            Message::SelectedPaths(paths)
                        }
                        Err(file_chooser::Error::Cancelled) => Message::Cancelled,
                        Err(why) => Message::LibraryPathOpenError(Arc::new(why)),
                    }
                });
            }

            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }

            // Cancel message for the Open Folder dialog
            Message::Cancelled => {}

            Message::ChangeTrack(id) => {
                // TODO: Make a proper artwork cache
                let (path, media_metadata) = self.library.from_id(id).unwrap();
                let uri = Url::from_file_path(path).unwrap();
                let now = Instant::now();

                if let Some(last) = self.list_last_clicked {
                    let elapsed = now.duration_since(last);
                    println!("{:?} {:?}", now, elapsed);
                    if elapsed <= Duration::from_millis(400) {
                        self.player.stop();
                        self.now_playing = Some(media_metadata.clone());
                        self.player.load(uri.as_str());
                        self.player.play();

                        if media_metadata.artwork_filename.is_some() {
                            let filename = media_metadata.artwork_filename.clone().unwrap();

                            let bytes = match self.load_artwork(&filename) {
                                Ok(bytes) => bytes,
                                Err(error) => {
                                    eprintln!("Failed to load album artwork: {:?}", error);
                                    Vec::new()
                                }
                            };
                            if bytes.len() > 0 {
                                self.album_artwork.insert(filename, bytes);
                            }
                        }
                    }
                }

                self.list_last_clicked = Some(now);
                println!("Change track: {:?}", media_metadata.title);
            }

            Message::KeyPressed(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
                if key == Key::Named(Named::Control) {
                    self.control_pressed = self.control_pressed + 1;
                }
                if key == Key::Named(Named::Shift) {
                    self.shift_pressed = self.shift_pressed + 1;
                }
            }

            Message::KeyReleased(key) => {
                if key == Key::Named(Named::Control) {
                    self.control_pressed = self.control_pressed - 1;
                }
                if key == Key::Named(Named::Shift) {
                    self.shift_pressed = self.shift_pressed - 1;
                }
            }

            Message::LibraryPathOpenError(why) => {
                eprintln!("{why}");
            }

            Message::ListSelectRow(id) => match self.control_pressed {
                0 => {
                    self.list_selected.clear();
                    self.list_selected.push(id);
                }
                1..2 => {
                    if self.list_selected.contains(&id) {
                        self.list_selected
                            .remove(self.list_selected.iter().position(|i| i == &id).unwrap());
                    } else {
                        self.list_selected.push(id);
                    }
                }
                _ => {}
            },

            // Handle scroll events from scrollable widgets
            Message::ListViewScroll(viewport) => {
                let viewport_height = viewport.bounds().height;
                self.list_view_scroll_offset = viewport.absolute_offset().y;

                self.list_start =
                    (self.list_view_scroll_offset / (self.list_row_height + 1.0)).floor() as usize;

                self.list_visible_row_count =
                    (viewport_height / (self.list_row_height + 1.0)).ceil() as usize;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::NewPlaylist => {}

            Message::Next => {
                println!("Next");
            }

            Message::Previous => {
                println!("Previous")
            }

            Message::Quit => {
                print!("Quit message sent");
                self.player.stop();
                process::exit(0);
            }

            Message::ReleaseSlider => {
                // TODO: Don't seek if the player statis isn't playing or paused
                self.dragging_progress_slider = false;
                match self.player.pipeline.seek_simple(
                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                    gst::ClockTime::from_seconds(self.playback_progress as u64),
                ) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Failed to seek: {:?}", err),
                };
            }

            Message::RemoveLibraryPath(path) => {
                let mut library_paths = self.config.library_paths.clone();
                library_paths.remove(&path);
                config_set!(library_paths, library_paths);
            }

            // Add selected paths from the Open dialog
            Message::SelectedPaths(paths) => {
                let mut library_paths = self.config.library_paths.clone();

                for path in paths {
                    library_paths.insert(path);
                }

                config_set!(library_paths, library_paths);
            }

            Message::SizeDecrease => {
                self.size_multiplier = self.size_multiplier - 2.0;
                if self.size_multiplier < 4.0 {
                    self.size_multiplier = 4.0;
                }

                self.update_list_row_height();
                state_set!(size_multiplier, self.size_multiplier);
            }

            Message::SizeIncrease => {
                self.size_multiplier = self.size_multiplier + 2.0;
                if self.size_multiplier > 20.0 {
                    self.size_multiplier = 20.0;
                }

                self.update_list_row_height();
                state_set!(size_multiplier, self.size_multiplier);
            }

            Message::SliderSeek(time) => {
                self.dragging_progress_slider = true;
                self.playback_progress = time;
            }

            // Handles GStreamer messages
            Message::Tick => {
                let bus = self.player.pipeline.bus().unwrap();
                while let Some(msg) = bus.pop() {
                    use gst::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => {
                            println!("End of stream.");
                            return self.update(Message::Next);
                        }
                        MessageView::Error(err) => {
                            eprintln!("Error: {}", err.error());
                            return self.update(Message::Next);
                        }
                        _ => (),
                    }
                }

                if !self.dragging_progress_slider {
                    if let Some(pos) = self.player.pipeline.query_position::<gst::ClockTime>() {
                        self.playback_progress = pos.mseconds() as f32 / 1000.0;
                    }
                }
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

            Message::ToggleListTextWrap(list_text_wrap) => {
                config_set!(list_text_wrap, list_text_wrap);
            }

            Message::TogglePlaying => {
                println!("Play/Pause");
                self.player.pause();
            }

            Message::UpdateComplete(library) => {
                self.library = library;
                match self.library.save(self.xdg_dirs.clone()) {
                    Ok(_) => {}
                    Err(e) => eprintln!("There was an error saving library data: {e}"),
                };
                self.is_updating = false;
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::UpdateLibrary => {
                // TODO: Clean this up
                if self.is_updating {
                    return Task::none();
                }
                self.is_updating = true;
                self.update_progress = 0.0;

                let library_paths = self.config.library_paths.clone();
                let xdg_dirs = self.xdg_dirs.clone();

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
                                .unwrap_or("")
                                .to_lowercase();
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

                        track_metadata.id = Some(digest(file_str));

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
                                track_metadata.duration = Some(duration.seconds() as f32);
                            }

                            // Cache artwork
                            if let Some(sample) = tags.get::<gst::tags::Image>() {
                                track_metadata.artwork_filename =
                                    cache_image(sample.get(), xdg_dirs.clone());
                            } else if let Some(sample) = tags.get::<gst::tags::PreviewImage>() {
                                track_metadata.artwork_filename =
                                    cache_image(sample.get(), xdg_dirs.clone());
                            }
                        } else {
                            // If there's no metadata just fill in the filename
                            track_metadata.title = Some(file.to_string_lossy().to_string());
                        }

                        // Update progress bar
                        update_progress = update_progress + 1.0;
                        if update_percent_old != (update_progress / update_total * 100.0).round() {
                            _ = tx.send(Message::UpdateProgress(
                                update_progress,
                                update_total,
                                (update_progress / update_total * 100.0).round(),
                            ));
                        }
                        update_percent_old = (update_progress / update_total * 100.0).round();
                    });

                    _ = tx.send(Message::UpdateProgress(update_total, update_total, 100.0));

                    std::thread::sleep(tokio::time::Duration::from_secs(1));
                    _ = tx.send(Message::UpdateComplete(library));
                });

                return cosmic::Task::stream(UnboundedReceiverStream::new(rx))
                    .map(cosmic::Action::App);
            }

            Message::UpdateProgress(update_progress, update_total, percent) => {
                self.update_progress = update_progress;
                self.update_total = update_total;
                self.update_percent = percent;
            }

            Message::WindowResized(size) => {
                let window_width = size.width;
                let window_height = size.height;
                state_set!(window_width, window_width);
                state_set!(window_height, window_height);
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
        Some(footer(self).into())
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

    /// Settings page content
    fn settings(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };

        let mut library_column = Column::new();

        library_column = library_column.push(
            Row::new()
                .push(
                    Column::new()
                        .push(
                            widget::button::text(fl!("add-location"))
                                .on_press(Message::AddLibraryDialog),
                        )
                        .width(Length::FillPortion(1))
                        .align_x(Alignment::Start),
                )
                .push(
                    Column::new()
                        .push(
                            widget::button::text(fl!("update-library"))
                                .on_press(Message::UpdateLibrary),
                        )
                        .width(Length::FillPortion(1))
                        .align_x(Alignment::End),
                )
                .width(Length::Fill),
        );

        let library_paths_length = self.config.library_paths.len() - 1;

        // Create library path rows
        for (i, path) in self.config.library_paths.iter().enumerate() {
            library_column = library_column.push(
                Row::new()
                    .width(Length::Fill)
                    .padding(space_xxs)
                    // Adds text
                    .push(widget::text::text(path.clone()).width(Length::FillPortion(1)))
                    // Adds delete button
                    .push(
                        widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                            .on_press(Message::RemoveLibraryPath(path.clone())),
                    ),
            );

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
                .title(fl!("list-view"))
                .add({
                    widget::settings::item::builder(fl!("wrap-text")).control(
                        toggler(self.config.list_text_wrap).on_toggle(Message::ToggleListTextWrap),
                    )
                })
                .into(),
            widget::settings::section()
                .title(fl!("library"))
                .add(library_column)
                .into(),
        ])
        .into()
    }

    /// Updates the cosmic config, in particular the theme
    fn update_config(&mut self) -> Task<cosmic::Action<Message>> {
        cosmic::command::set_theme(self.config.app_theme.theme())
    }

    /// Calculate the playback time
    pub fn display_playback_progress(&self) -> String {
        let minutes = (self.playback_progress / 60.0) as u32;
        let seconds = f32::trunc(self.playback_progress) as u32 - (minutes * 60);
        let value = format!("{}:{:02}", minutes, seconds);
        value
    }

    pub fn display_time_left(&self) -> String {
        if self.now_playing.is_some() {
            let now_playing = self.now_playing.clone().unwrap();
            let duration = now_playing.duration.unwrap_or(0.0);

            let mut time_left = duration - self.playback_progress;
            if time_left < 0.0 {
                time_left = 0.0;
            }
            if time_left > duration {
                time_left = duration;
            }

            let minutes = (time_left / 60.0) as u32;
            let seconds = f32::trunc(time_left) as u32 - (minutes * 60);

            return format!("-{}:{:02}", minutes, seconds);
        }

        String::from("-0.00")
    }

    pub fn load_artwork(&self, filename: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut path: PathBuf = self.artwork_dir.clone().unwrap();
        path = path.join(&filename);

        let bytes = fs::read(&path)?;

        Ok(bytes)
    }

    pub fn get_artwork(&self, filename: String) -> Option<&Vec<u8>> {
        self.album_artwork.get(&filename)
    }

    pub fn update_list_row_height(&mut self) {
        self.list_row_height = 5.0 * self.size_multiplier;
    }
}

/// Flags passed into the app
#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub state_handler: Option<cosmic_config::Config>,
    pub state: State,
}

/// The page to display in the application.
pub enum Page {
    Library,
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
    NewPlaylist,
    Quit,
    Settings,
    SizeDecrease,
    SizeIncrease,
    UpdateLibrary,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::NewPlaylist => Message::NewPlaylist,
            MenuAction::Quit => Message::Quit,
            MenuAction::SizeDecrease => Message::SizeDecrease,
            MenuAction::SizeIncrease => Message::SizeIncrease,
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::UpdateLibrary => Message::UpdateLibrary,
        }
    }
}

// TODO: Clean this up
fn cache_image(sample: gst::Sample, xdg_dirs: BaseDirectories) -> Option<String> {
    let buffer = match sample.buffer() {
        Some(b) => b,
        None => return None,
    };

    let caps = match sample.caps() {
        Some(c) => c,
        None => return None,
    };

    let mime = caps
        .structure(0)
        .and_then(|s| s.name().split('/').nth(1))
        .unwrap_or("jpg");

    let map = buffer.map_readable().ok();
    let hash = digest(map.as_ref().unwrap().as_slice());
    let file_name = format!("{hash}.{mime}");
    let full_path = match xdg_dirs.place_cache_file(format!("artwork/{file_name}")) {
        Ok(full_path) => full_path,
        Err(_) => return None,
    };

    if !Path::new(&full_path).exists() {
        let mut file = match File::create(full_path) {
            Ok(file) => file,
            Err(_) => return None,
        };

        match file.write_all(map.unwrap().as_slice()) {
            Ok(()) => (),
            Err(err) => eprintln!("Cannot save album artwork: {:?}", err),
        }
    }
    Some(file_name)
}
