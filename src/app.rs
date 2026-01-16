// SPDX-License-Identifier: GPL-3.0

use crate::config::{AppTheme, CONFIG_VERSION, Config, State};
use crate::fl;
use crate::footer::footer;
use crate::key_bind::key_binds;
use crate::library::Library;
use crate::library::MediaMetaData;
use crate::menu::menu_bar;
use crate::page::empty_library;
use crate::page::list_view;
use crate::page::loading;
use crate::player::Player;
use crate::playlist::Playlist;
use cosmic::iced_widget::scrollable;
use cosmic::prelude::*;
use cosmic::{
    app::context_drawer,
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme,
    dialog::file_chooser,
    iced::{
        self, Alignment, Length, Size, Subscription,
        alignment::{Horizontal, Vertical},
        event::{self, Event},
        keyboard::{Event as KeyEvent, Key, Modifiers, key::Named},
        window::Event as WindowEvent,
    },
    theme,
    widget::{
        self, Column,
        about::About,
        menu::{self, Action as WidgetMenuAction},
        nav_bar, row, settings, text, toggler,
    },
};
use gst::prelude::ElementExt;
use gst::prelude::ElementExtManual;
use gstreamer as gst;
use gstreamer_pbutils as pbutils;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
    /// The about page this app.
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
    pub state: crate::config::State,

    pub rdnn_xdg_dirs: BaseDirectories,
    app_xdg_dirs: BaseDirectories,

    pub library: Library,

    pub is_updating: bool,
    pub playback_progress: f32,
    pub update_progress: f32,
    pub update_total: f32,
    pub update_percent: f32,
    pub update_progress_display: String,

    initial_load_complete: bool,

    player: Player,

    dialog_pages: DialogPages,

    pub now_playing: Option<MediaMetaData>,
    pub now_playing_handle: Option<widget::image::Handle>,
    pub artwork_dir: Option<PathBuf>,
    album_artwork: HashMap<String, Vec<u8>>,
    dragging_progress_slider: bool,

    size_multiplier: f32,
    pub list_row_height: f32,
    pub list_view_scroll_offset: f32,
    pub list_start: usize,
    pub list_visible_row_count: usize,
    pub list_selected: Vec<String>,
    list_last_clicked: Option<Instant>,

    control_pressed: u8,
    shift_pressed: u8,

    pub playlists: Vec<crate::playlist::Playlist>,
    view_playlist: Option<u32>,
    audio_playlist: Option<u32>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    AddLibraryCancel,
    AddLibraryDialog,
    AppTheme(AppTheme),
    ChangeTrack(String),
    DeletePlaylist,
    DialogCancel,
    DialogComplete,
    KeyPressed(Modifiers, Key),
    KeyReleased(Key),
    LaunchUrl(String),
    LibraryPathOpenError(Arc<file_chooser::Error>),
    ListSelectRow(String),
    ListViewScroll(scrollable::Viewport),
    ListViewSort(SortBy),
    MoveNavDown,
    MoveNavUp,
    NewPlaylist,
    Next,
    PeriodicLibraryUpdate(HashMap<PathBuf, MediaMetaData>),
    Previous,
    Quit,
    ReleaseSlider,
    RemoveLibraryPath(String),
    RenamePlaylist,
    SelectedPaths(Vec<String>),
    SliderSeek(f32),
    Tick,
    ToggleContextPage(ContextPage),
    ToggleListTextWrap(bool),
    TogglePlaying,
    UpdateComplete(Library),
    UpdateConfig(Config),
    UpdateDialog(DialogPage),
    UpdateLibrary,
    UpdateProgress(f32, f32, f32),
    WindowResized(Size),
    ZoomIn,
    ZoomOut,
}

/// Unique identifier in RDNN (reverse domain name notation) format.
pub const APP_ID: &'static str = "com.github.LotusPetal392.ethereal-waves";

const NEW_PLAYLIST_INPUT_ID: &str = "new_playlist_input_id";
const RENAME_PLAYLIST_INPUT_ID: &str = "rename_playlist_input_id";

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = Flags;

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = APP_ID;

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let nav = nav_bar::Model::default();

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
            config: cosmic_config::Config::new(APP_ID, CONFIG_VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            app_theme_labels: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            config_handler: _flags.config_handler,
            state_handler: _flags.state_handler,
            state: _flags.state.clone(),
            rdnn_xdg_dirs: xdg::BaseDirectories::with_prefix(APP_ID),
            app_xdg_dirs: xdg::BaseDirectories::with_prefix("ethereal-waves"),
            initial_load_complete: false,
            library: Library::new(),
            is_updating: false,
            playback_progress: 0.0,
            update_progress: 0.0,
            update_total: 0.0,
            update_percent: 0.0,
            update_progress_display: "0".into(),
            dragging_progress_slider: false,
            player: Player::new(),
            dialog_pages: DialogPages::new(),
            now_playing: None,
            now_playing_handle: None,
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
            playlists: Vec::new(),
            view_playlist: None,
            audio_playlist: None,
        };

        // Create a startup command that sets the window title.
        let update_title = app.update_title();

        // Load the master library and playlists
        let load_data = app.load_data();

        // Build out artwork cache directory
        app.artwork_dir = app.rdnn_xdg_dirs.get_cache_home();
        app.artwork_dir = app.artwork_dir.map(|p| p.join("artwork"));

        app.update_list_row_height();

        (app, Task::batch([update_title, load_data]))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu_bar(self.is_updating, self.view_playlist, &self.key_binds);
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
        if self.initial_load_complete == false {
            return loading::content().into();
        }

        let playlist = self
            .playlists
            .iter()
            .find(|p| Some(p.id()) == self.view_playlist);

        let content: Column<_> = match playlist {
            Some(p) if p.is_library() && p.tracks().is_empty() => empty_library::content(),
            Some(p) => list_view::content(self, p),
            None => empty_library::content(),
        };

        widget::container(widget::column().push(content))
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::NewPlaylist(name) => {
                let complete_maybe = if name.is_empty() {
                    None
                } else if name.trim().is_empty() {
                    None
                } else {
                    Some(Message::DialogComplete)
                };

                let dialog = widget::dialog()
                    .title(fl!("new-playlist"))
                    .primary_action(
                        widget::button::suggested(fl!("create")).on_press_maybe(complete_maybe),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(widget::column::with_children(vec![
                        widget::text_input(fl!("untitled-playlist"), name)
                            .id(widget::Id::new(NEW_PLAYLIST_INPUT_ID))
                            .on_input(move |name| {
                                Message::UpdateDialog(DialogPage::NewPlaylist(name))
                            })
                            .into(),
                    ]));

                dialog
            }

            DialogPage::RenamePlaylist { id, name } => {
                let complete_maybe = if name.is_empty() {
                    None
                } else if name.trim().is_empty() {
                    None
                } else {
                    Some(Message::DialogComplete)
                };

                let dialog = widget::dialog()
                    .title(fl!("rename-playlist"))
                    .primary_action(
                        widget::button::suggested(fl!("rename")).on_press_maybe(complete_maybe),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(widget::column::with_children(vec![
                        widget::text_input("", name)
                            .on_input(move |name| {
                                Message::UpdateDialog(DialogPage::RenamePlaylist {
                                    id: *id,
                                    name: name,
                                })
                            })
                            .id(widget::Id::new(RENAME_PLAYLIST_INPUT_ID))
                            .into(),
                    ]));
                dialog
            }

            DialogPage::DeletePlaylist(id) => {
                let playlist = self.playlists.iter().find(|p| p.id() == *id);

                let dialog = widget::dialog()
                    .title(fl!("delete-playlist"))
                    .icon(widget::icon::from_name("dialog-warning").size(64))
                    .body(format!("{} {}?", fl!("delete"), playlist.unwrap().name()))
                    .primary_action(
                        widget::button::suggested(fl!("yes")).on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(widget::column::with_children(vec![
                        widget::text(fl!("delete-warning")).into(),
                    ]));
                dialog
            }
        };

        Some(dialog.into())
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
            self.core().watch_config::<Config>(APP_ID).map(|update| {
                // for why in update.errors {
                //     tracing::error!(?why, "app config error");
                // }

                Message::UpdateConfig(update.config)
            }),
        ];

        if self.now_playing.is_some() {
            subscriptions
                .push(iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick));
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
            // Open dialog for adding library locations
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
                        Err(file_chooser::Error::Cancelled) => Message::AddLibraryCancel,
                        Err(why) => Message::LibraryPathOpenError(Arc::new(why)),
                    }
                });
            }

            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }

            // Cancel message for the Open Folder dialog
            Message::AddLibraryCancel => {}

            Message::ChangeTrack(id) => {
                // TODO: Make a proper artwork cache
                if !self.library.from_id(id.clone()).is_some() {
                    return Task::none();
                }
                let (path, media_metadata) = self.library.from_id(id).unwrap();
                let uri = Url::from_file_path(path).unwrap();
                let now = Instant::now();

                if let Some(last) = self.list_last_clicked {
                    let elapsed = now.duration_since(last);

                    if elapsed <= Duration::from_millis(400) {
                        println!("Change track: {:?}", media_metadata.title);
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
                                self.album_artwork.insert(filename, bytes.clone());
                                self.now_playing_handle =
                                    Some(widget::image::Handle::from_bytes(bytes));
                            }
                        } else {
                            self.now_playing_handle = None;
                        }
                    }
                }

                self.list_last_clicked = Some(now);
            }

            Message::DialogCancel => {
                let _ = self.dialog_pages.pop_front();
            }

            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::NewPlaylist(name) => {
                            let playlist = Playlist::new(name);
                            self.playlists.push(playlist.clone());

                            let divider = self.playlists.len() == 2;
                            let playlist_id = playlist.id();

                            self.nav
                                .insert()
                                .text(playlist.name().to_string())
                                .icon(widget::icon::from_name("playlist-symbolic"))
                                .data(Page::Playlist(playlist.id()))
                                .divider_above(divider);

                            let _ = self.save_playlists(Some(playlist_id));
                        }

                        DialogPage::RenamePlaylist { id, name } => {
                            // Update playlist
                            if let Some(playlist) = self.playlists.iter_mut().find(|p| p.id() == id)
                            {
                                playlist.set_name(name.clone());
                            }

                            // Update nav text
                            let entity = self.nav.iter().find(|&e| {
                                self.nav.data::<Page>(e).map_or(false, |page| match page {
                                    Page::Playlist(playlist_id) => *playlist_id == id,
                                    _ => false,
                                })
                            });

                            if let Some(entity) = entity {
                                self.nav.text_set(entity, name);
                            }

                            let _ = self.save_playlists(Some(id));
                        }

                        DialogPage::DeletePlaylist(id) => {
                            let _ = self.delete_playlist(id);
                        }
                    };
                };
            }

            Message::KeyPressed(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
                if key == Key::Named(Named::Control) && self.control_pressed < 2 {
                    self.control_pressed += 1;
                }
                if key == Key::Named(Named::Shift) && self.shift_pressed < 2 {
                    self.shift_pressed += 1;
                }

                if self.dialog_pages.front().is_some() {
                    if key == Key::Named(Named::Escape) {
                        return self.update(Message::DialogCancel);
                    }

                    match self.dialog_pages.front().unwrap() {
                        DialogPage::NewPlaylist(name) => {
                            if key == Key::Named(Named::Enter) && name.len() > 0 {
                                return self.update(Message::DialogComplete);
                            }
                        }
                        DialogPage::RenamePlaylist { id, name } => {
                            let _ = id;
                            if key == Key::Named(Named::Enter) && name.len() > 0 {
                                return self.update(Message::DialogComplete);
                            }
                        }
                        DialogPage::DeletePlaylist(_) => {
                            if key == Key::Named(Named::Enter) {
                                return self.update(Message::DialogComplete);
                            }
                        }
                    }
                }
            }

            Message::KeyReleased(key) => {
                if key == Key::Named(Named::Control) {
                    self.control_pressed = self.control_pressed.saturating_sub(1);
                }
                if key == Key::Named(Named::Shift) {
                    self.shift_pressed = self.shift_pressed.saturating_sub(1);
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

            Message::ListViewSort(sort_by) => {
                let sort_direction = match self.state.sort_direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
                state_set!(sort_by, sort_by.clone());
                state_set!(sort_direction, sort_direction.clone());

                if self.view_playlist.is_some() {
                    if let Some(i) = self
                        .playlists
                        .iter()
                        .position(|p| p.id() == self.view_playlist.unwrap())
                    {
                        self.playlists[i].sort(sort_by, sort_direction);
                    };
                }
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            // Kick off the New Playlist dialog
            Message::NewPlaylist => {
                self.dialog_pages
                    .push_back(DialogPage::NewPlaylist(String::new()));
                return widget::text_input::focus(widget::Id::new(NEW_PLAYLIST_INPUT_ID));
            }

            // Kick off the Rename Playlist dialog
            Message::RenamePlaylist => match self.nav.data(self.nav.active()) {
                Some(Page::Playlist(id)) => {
                    let name = self.nav.text(self.nav.active()).unwrap_or("");
                    self.dialog_pages.push_back(DialogPage::RenamePlaylist {
                        id: *id,
                        name: name.into(),
                    });
                    return widget::text_input::focus(widget::Id::new(RENAME_PLAYLIST_INPUT_ID));
                }
                _ => {}
            },

            // Kick off the delete playlist dialog
            Message::DeletePlaylist => {
                if let Some(Page::Playlist(id)) = self.nav.data(self.nav.active()) {
                    if let Some(p) = self.playlists.iter().find(|p| p.id() == *id) {
                        if !p.is_library() {
                            self.dialog_pages.push_back(DialogPage::DeletePlaylist(*id));
                        }
                    }
                }
            }

            Message::MoveNavUp | Message::MoveNavDown => {
                self.move_active_nav(if matches!(message, Message::MoveNavUp) {
                    -1
                } else {
                    1
                });

                let order = self.nav_order();

                state_set!(playlist_nav_order, order);
            }

            Message::Next => {
                println!("Next");
            }

            Message::PeriodicLibraryUpdate(media) => {
                self.library.media = media;
                let _ = self.library.save(&self.app_xdg_dirs);

                // Update the library playlist with new data
                if let Some(lib_playlist) = self.playlists.iter_mut().find(|p| p.is_library()) {
                    lib_playlist.clear(); // Clear existing tracks
                    for (path, metadata) in &self.library.media {
                        lib_playlist.push((path.clone(), metadata.clone()));
                    }
                    lib_playlist.sort(
                        self.state.sort_by.clone(),
                        self.state.sort_direction.clone(),
                    );
                }
            }

            Message::Previous => {
                println!("Previous");
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
                match self.library.save(&self.app_xdg_dirs) {
                    Ok(_) => {}
                    Err(e) => eprintln!("There was an error saving library data: {e}"),
                };
                self.is_updating = false;

                // Update the library playlist with new data
                if let Some(lib_playlist) = self.playlists.iter_mut().find(|p| p.is_library()) {
                    lib_playlist.clear(); // Clear existing tracks
                    for (path, metadata) in &self.library.media {
                        lib_playlist.push((path.clone(), metadata.clone()));
                    }
                    lib_playlist.sort(
                        self.state.sort_by.clone(),
                        self.state.sort_direction.clone(),
                    );
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::UpdateDialog(dialog_page) => match dialog_page {
                DialogPage::NewPlaylist(name) => {
                    self.dialog_pages
                        .update_front(DialogPage::NewPlaylist(name));
                }

                DialogPage::RenamePlaylist { id, name } => {
                    self.dialog_pages
                        .update_front(DialogPage::RenamePlaylist { id: id, name: name });
                }

                DialogPage::DeletePlaylist(id) => {
                    self.dialog_pages
                        .update_front(DialogPage::DeletePlaylist(id));
                }
            },

            Message::UpdateLibrary => {
                if self.is_updating {
                    return Task::none();
                }
                self.is_updating = true;
                self.update_progress = 0.0;

                let library_paths = self.config.library_paths.clone();
                let xdg_dirs = self.app_xdg_dirs.clone();

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

                    let mut update_progress: f32 = 0.0;
                    let update_total: f32 = library.media.len() as f32;

                    let mut last_progress_update: Instant = std::time::Instant::now();
                    let update_progress_interval: Duration = std::time::Duration::from_millis(200);

                    let mut last_library_update: Instant = std::time::Instant::now();
                    let update_library_interval: Duration = std::time::Duration::from_secs(10);

                    let mut entries: Vec<(PathBuf, MediaMetaData)> =
                        library.media.into_iter().collect();

                    let mut completed_entries: HashMap<PathBuf, MediaMetaData> = HashMap::new();

                    entries.iter_mut().for_each(|(file, track_metadata)| {
                        let discoverer =
                            match pbutils::Discoverer::new(gst::ClockTime::from_seconds(5)) {
                                Ok(discoverer) => discoverer,
                                Err(error) => panic!("Failed to create discoverer: {:?}", error),
                            };

                        let file_str = match file.to_str() {
                            Some(file_str) => file_str,
                            None => "",
                        };

                        let uri = Url::from_file_path(file_str).unwrap();

                        let info = match discoverer.discover_uri(&uri.as_str()) {
                            Ok(info) => info,
                            Err(err) => {
                                eprintln!("Failed to read metadata from {}: {}", file_str, err);
                                return; // Skip this file and move on
                            }
                        };

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

                        completed_entries.insert(file.clone(), track_metadata.clone());

                        // Update progress bar
                        // let mut progress: f32 = update_progress;
                        update_progress += 1.0;
                        let now = std::time::Instant::now();
                        if now.duration_since(last_progress_update) >= update_progress_interval {
                            last_progress_update = now;
                            _ = tx.send(Message::UpdateProgress(
                                update_progress,
                                update_total,
                                update_progress / update_total * 100.0,
                            ));
                        }

                        // Send periodic library updates
                        if now.duration_since(last_library_update) >= update_library_interval {
                            last_library_update = now;
                            _ = tx.send(Message::PeriodicLibraryUpdate(completed_entries.clone()));
                        }
                    });

                    // Convert back to HashMap
                    library.media = entries.into_iter().collect();

                    // Remove anything without an id
                    library.media.retain(|_, v| v.id.is_some());

                    _ = tx.send(Message::UpdateProgress(update_total, update_total, 100.0));
                    _ = tx.send(Message::UpdateComplete(library));
                });

                return cosmic::Task::stream(UnboundedReceiverStream::new(rx))
                    .map(cosmic::Action::App);
            }

            Message::UpdateProgress(update_progress, update_total, percent) => {
                self.update_progress = update_progress;
                self.update_total = update_total;
                self.update_percent = percent;
                self.update_progress_display = format!(
                    "{} {}/{} ({:.2}%)",
                    fl!("updating-library"),
                    update_progress,
                    update_total,
                    percent
                )
            }

            Message::WindowResized(size) => {
                let window_width = size.width;
                let window_height = size.height;
                state_set!(window_width, window_width);
                state_set!(window_height, window_height);
            }

            Message::ZoomIn => {
                self.size_multiplier = self.size_multiplier + 2.0;
                if self.size_multiplier > 30.0 {
                    self.size_multiplier = 30.0;
                }

                self.update_list_row_height();
                state_set!(size_multiplier, self.size_multiplier);
            }

            Message::ZoomOut => {
                self.size_multiplier = self.size_multiplier - 2.0;
                if self.size_multiplier < 4.0 {
                    self.size_multiplier = 4.0;
                }

                self.update_list_row_height();
                state_set!(size_multiplier, self.size_multiplier);
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        if let Some(Page::Playlist(pid)) = self.nav.data(id) {
            self.view_playlist = Some(*pid);
        }

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

        let page = self.nav.text(self.nav.active());

        if page.is_some() {
            window_title.push_str(" — ");
            window_title.push_str(page.unwrap());
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

        let mut library_column = widget::column();

        library_column = library_column.push(
            row()
                .push(
                    widget::column()
                        .push(
                            widget::button::text(fl!("add-location"))
                                .on_press(Message::AddLibraryDialog),
                        )
                        .width(Length::FillPortion(1))
                        .align_x(Alignment::Start),
                )
                .push(
                    widget::column()
                        .push(
                            widget::button::text(fl!("update-library"))
                                .on_press(Message::UpdateLibrary),
                        )
                        .width(Length::FillPortion(1))
                        .align_x(Alignment::End),
                )
                .width(Length::Fill),
        );

        let library_paths_length = self.config.library_paths.len().saturating_sub(1);

        // Create library path rows
        for (i, path) in self.config.library_paths.iter().enumerate() {
            library_column = library_column.push(
                row()
                    .width(Length::Fill)
                    .padding(space_xxs)
                    // Adds text
                    .push(text::text(path.clone()).width(Length::FillPortion(1)))
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

        settings::view_column(vec![
            settings::section()
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
            settings::section()
                .title(fl!("list-view"))
                .add({
                    settings::item::builder(fl!("wrap-text")).control(
                        toggler(self.config.list_text_wrap).on_toggle(Message::ToggleListTextWrap),
                    )
                })
                .into(),
            settings::section()
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
        format!("{}:{:02}", minutes, seconds)
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

    pub fn update_list_row_height(&mut self) {
        self.list_row_height = 5.0 * self.size_multiplier;
    }

    /// Load library and playlists
    pub fn load_data(&mut self) -> Task<cosmic::Action<Message>> {
        // Load library
        let media: HashMap<PathBuf, MediaMetaData> =
            match AppModel::load_library(&self.app_xdg_dirs) {
                Ok(media) => media,
                Err(err) => {
                    eprintln!("Failed to load library: {err}");
                    HashMap::new()
                }
            };
        self.library.media = media;

        // Setup library playlist
        let mut library_playlist = Playlist::library();
        for (path, metadata) in &self.library.media {
            library_playlist.push((path.clone(), metadata.clone()));
        }

        library_playlist.sort(
            self.state.sort_by.clone(),
            self.state.sort_direction.clone(),
        );

        // Load user playlists
        let mut playlists = vec![library_playlist];
        playlists.extend(self.load_playlists().unwrap_or_default());
        self.playlists = playlists;

        println!("There are {} playlists loaded.", self.playlists.len());

        // Decide nav order
        let items: Vec<NavPlaylistItem> = if !self.state.playlist_nav_order.is_empty() {
            // Start with saved order
            let mut ordered_items: Vec<NavPlaylistItem> = self
                .state
                .playlist_nav_order
                .iter()
                .filter_map(|pid| {
                    self.playlists
                        .iter()
                        .find(|p| p.id() == *pid)
                        .map(|p| NavPlaylistItem {
                            id: *pid,
                            name: p.name().to_string(),
                        })
                })
                .collect();

            // Add any playlists that aren't in the saves order
            let ordered_ids: HashSet<_> = ordered_items.iter().map(|item| item.id).collect();
            for playlist in self.playlists.iter().filter(|p| !p.is_library()) {
                if !ordered_ids.contains(&playlist.id()) {
                    ordered_items.push(NavPlaylistItem {
                        id: playlist.id(),
                        name: playlist.name().to_string(),
                    });
                }
            }

            ordered_items
        } else {
            self.playlists
                .iter()
                .filter(|p| !p.is_library())
                .map(|p| NavPlaylistItem {
                    id: p.id(),
                    name: p.name().to_string(),
                })
                .collect()
        };

        // Decide what should be active
        let active_id = self
            .view_playlist
            .or_else(|| {
                self.playlists
                    .iter()
                    .find(|p| p.is_library())
                    .map(|p| p.id())
            })
            .unwrap();

        // Rebuild nav once
        self.rebuild_nav_from_order(items, active_id);

        self.initial_load_complete = true;
        Task::none()
    }

    /// Load library.json file if it exists
    pub fn load_library(
        xdg_dirs: &BaseDirectories,
    ) -> anyhow::Result<HashMap<PathBuf, MediaMetaData>> {
        let mut media: HashMap<PathBuf, MediaMetaData> = xdg_dirs
            .get_data_file("library.json")
            .map(|path| {
                let content = fs::read_to_string(path)?;
                Ok::<_, anyhow::Error>(serde_json::from_str(&content)?)
            })
            .transpose()?
            .unwrap_or_default();

        // Remove any entry without an id
        media.retain(|_, v| v.id.is_some());

        Ok(media)
    }

    /// Load playlist files
    pub fn load_playlists(&self) -> anyhow::Result<Vec<Playlist>> {
        // Make sure playlist path exists
        let playlist_path = self.app_xdg_dirs.create_data_directory("playlists")?;

        let mut playlists: Vec<Playlist> = Vec::new();

        // Read in all the json files in the directory
        for file in fs::read_dir(playlist_path)? {
            let file = file?;
            let file_path = file.path();

            println!("file found: {}", file_path.to_string_lossy());

            if file_path.extension().and_then(|e| e.to_str()) == Some("json") {
                let contents = fs::read_to_string(&file_path)?;
                playlists.push(serde_json::from_str(&contents)?);
            }
        }

        Ok(playlists)
    }

    fn save_playlists(&self, id: Option<u32>) -> anyhow::Result<()> {
        let playlist_path = self.app_xdg_dirs.create_data_directory("playlists")?;

        // Make sure path exists
        let _ = fs::create_dir_all(&playlist_path);

        if id.is_some() {
            let filename = format!("{}.json", id.unwrap());
            let file_path = playlist_path.join(&filename);

            if let Some(playlist) = self.playlists.iter().find(|p| p.id() == id.unwrap()) {
                let json_data =
                    serde_json::to_string(playlist).expect("Failed to serialize playlist");
                let mut file = File::create(file_path).expect("Failed to create playlist file");
                file.write_all(json_data.as_bytes())
                    .expect("Failed to write JSON data to file");
            }
        }

        Ok(())
    }

    /// Delete playlist from filesystem, playlists, and nav area
    pub fn delete_playlist(&mut self, id: u32) -> anyhow::Result<()> {
        // Delete from filesystem
        let playlist = self.playlists.iter().find(|p| p.id() == id);
        let filename = format!("{}.json", playlist.unwrap().id());
        let mut file_path = self.app_xdg_dirs.create_data_directory("playlists")?;
        file_path = file_path.join(filename);

        fs::remove_file(file_path)?;

        // Delete from nav
        let ids: Vec<_> = self.nav.iter().collect();

        if let Some(entity) = ids.iter().copied().find(|e| {
            self.nav
                .data::<Page>(*e)
                .and_then(|p| match p {
                    Page::Playlist(pid) => self
                        .playlists
                        .iter()
                        .find(|pl| pl.id() == *pid && pl.is_library()),
                })
                .is_some()
        }) {
            self.nav.activate(entity);
            self.view_playlist = self.nav.data(entity).and_then(|p| {
                if let Page::Playlist(id) = p {
                    Some(*id)
                } else {
                    None
                }
            });
        }

        if let Some(entity) = ids.iter().copied().find(|e| {
            self.nav.data::<Page>(*e).map_or(false, |p| match p {
                Page::Playlist(pid) => *pid == id,
            })
        }) {
            self.nav.remove(entity);
        }

        Ok(())
    }

    fn rebuild_nav_from_order(&mut self, items: Vec<NavPlaylistItem>, activate_id: u32) {
        self.nav.clear();

        let library_id = self.playlists.iter().find(|p| p.is_library()).unwrap().id();

        self.nav
            .insert()
            .text(fl!("library"))
            .data(Page::Playlist(library_id))
            .icon(widget::icon::from_name("folder-music-symbolic"));

        for (i, item) in items.iter().enumerate() {
            let Some(_) = self.playlists.iter().find(|p| p.id() == item.id) else {
                continue;
            };

            self.nav
                .insert()
                .text(item.name.clone())
                .icon(widget::icon::from_name("playlist-symbolic"))
                .data(Page::Playlist(item.id))
                .divider_above(i == 0);
        }

        let nav_id_to_activate = self
            .nav
            .iter()
            .find_map(|id| match self.nav.data::<Page>(id) {
                Some(Page::Playlist(pid)) if *pid == activate_id => Some(id),
                _ => None,
            });

        if let Some(id) = nav_id_to_activate {
            self.nav.activate(id);
            self.view_playlist = Some(activate_id);
        }

        self.nav_order();
    }

    /// Swap positions of nav items
    fn move_active_nav(&mut self, direction: i32) {
        let active = self.nav.active();

        let Some(Page::Playlist(active_id)) = self.nav.data::<Page>(active) else {
            return;
        };

        let active_playlist = self
            .playlists
            .iter()
            .find(|p| p.id() == *active_id)
            .unwrap();

        if active_playlist.is_library() {
            return;
        }

        let mut items: Vec<_> = self
            .nav
            .iter()
            .filter_map(|nav_id| {
                self.nav.data::<Page>(nav_id).and_then(|p| match p {
                    Page::Playlist(pid) => self
                        .playlists
                        .iter()
                        .find(|pl| pl.id() == *pid && !pl.is_library())
                        .map(|pl| NavPlaylistItem {
                            id: *pid,
                            name: pl.name().to_string(),
                        }),
                })
            })
            .collect();

        let idx = items.iter().position(|p| p.id == *active_id).unwrap();

        let new_idx = match direction {
            -1 if idx > 0 => idx - 1,
            1 if idx + 1 < items.len() => idx + 1,
            _ => return,
        };

        items.swap(idx, new_idx);

        self.rebuild_nav_from_order(items, *active_id);
    }

    fn nav_order(&mut self) -> Vec<u32> {
        self.nav
            .iter()
            .filter_map(|id| {
                self.nav.data::<Page>(id).and_then(|page| match page {
                    Page::Playlist(pid) => self
                        .playlists
                        .iter()
                        .find(|p| p.id() == *pid && !p.is_library())
                        .map(|_| *pid),
                })
            })
            .collect()
    }
}

#[derive(Clone)]
struct NavPlaylistItem {
    id: u32,
    name: String,
}

/// Flags passed into the app
#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub state_handler: Option<cosmic_config::Config>,
    pub state: State,
}

/// The page to display in the application.
#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    Playlist(u32),
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
    DeletePlaylist,
    MoveNavDown,
    MoveNavUp,
    NewPlaylist,
    Quit,
    RenamePlaylist,
    Settings,
    UpdateLibrary,
    ZoomIn,
    ZoomOut,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::DeletePlaylist => Message::DeletePlaylist,
            MenuAction::MoveNavDown => Message::MoveNavDown,
            MenuAction::MoveNavUp => Message::MoveNavUp,
            MenuAction::NewPlaylist => Message::NewPlaylist,
            MenuAction::RenamePlaylist => Message::RenamePlaylist,
            MenuAction::Quit => Message::Quit,
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::UpdateLibrary => Message::UpdateLibrary,
            MenuAction::ZoomIn => Message::ZoomIn,
            MenuAction::ZoomOut => Message::ZoomOut,
        }
    }
}

// Saves album artwork to files, no duplicates
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

#[derive(Clone, Debug)]
pub enum DialogPage {
    NewPlaylist(String),
    RenamePlaylist { id: u32, name: String },
    DeletePlaylist(u32),
}

pub struct DialogPages {
    pages: VecDeque<DialogPage>,
}

impl Default for DialogPages {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogPages {
    pub const fn new() -> Self {
        Self {
            pages: VecDeque::new(),
        }
    }

    pub fn front(&self) -> Option<&DialogPage> {
        self.pages.front()
    }

    pub fn push_back(&mut self, page: DialogPage) {
        self.pages.push_back(page);
    }

    #[must_use]
    pub fn pop_front(&mut self) -> Option<DialogPage> {
        let page = self.pages.pop_front()?;
        Some(page)
    }

    pub fn update_front(&mut self, page: DialogPage) {
        if !self.pages.is_empty() {
            self.pages[0] = page;
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SortBy {
    Artist,
    Album,
    Title,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlaylistKind {
    Library,
    User,
}
