// SPDX-License-Identifier: GPL-3.0

use crate::app::{AppModel, MenuAction, Message, RepeatMode};
use crate::fl;
use cosmic::{Apply, Element, iced::Length, widget::menu};

pub fn menu_bar<'a>(app: &AppModel) -> Element<'a, Message> {
    let has_playlist = app.view_playlist.is_some();

    let repeat_one = if app.state.repeat_mode == RepeatMode::One {
        true
    } else {
        false
    };

    let repeat_all = if app.state.repeat_mode == RepeatMode::All {
        true
    } else {
        false
    };

    let selected_playlist = app
        .playlists
        .iter()
        .find(|p| p.id() == app.view_playlist.unwrap())
        .unwrap();

    let mut selected_playlist_list = Vec::new();
    let mut now_playing_playlist_list = Vec::new();

    let selected_count: usize = if app.view_playlist.is_some() {
        app.playlists
            .iter()
            .find(|p| p.id() == app.view_playlist.unwrap())
            .unwrap()
            .selected_iter()
            .count()
    } else {
        0
    };

    // Add ordered playlists
    app.state.playlist_nav_order.iter().for_each(|p| {
        let playlist = app.playlists.iter().find(|playlist| playlist.id() == *p);
        if playlist.is_some() {
            selected_playlist_list.push(menu::Item::Button(
                playlist.unwrap().name().to_string(),
                None,
                MenuAction::AddSelectedToPlaylist(playlist.unwrap().id()),
            ));
            if app.now_playing.is_some() {
                now_playing_playlist_list.push(menu::Item::Button(
                    playlist.unwrap().name().to_string(),
                    None,
                    MenuAction::AddNowPlayingToPlaylist(playlist.unwrap().id()),
                ));
            }
        }
    });
    // Add unordered playlists
    app.playlists
        .iter()
        .filter(|p| !p.is_library() && !app.state.playlist_nav_order.contains(&p.id()))
        .for_each(|p| {
            selected_playlist_list.push(menu::Item::Button(
                p.name().to_string(),
                None,
                MenuAction::AddSelectedToPlaylist(p.id()),
            ));
            if app.now_playing.is_some() {
                now_playing_playlist_list.push(menu::Item::Button(
                    p.name().to_string(),
                    None,
                    MenuAction::AddNowPlayingToPlaylist(p.id()),
                ));
            }
        });

    menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    if selected_count > 0 {
                        menu::Item::Button(fl!("track-info"), None, MenuAction::TrackInfoPanel)
                    } else {
                        menu::Item::ButtonDisabled(
                            fl!("track-info"),
                            None,
                            MenuAction::TrackInfoPanel,
                        )
                    },
                    menu::Item::Divider,
                    if app.is_updating {
                        menu::Item::ButtonDisabled(
                            fl!("update-library"),
                            None,
                            MenuAction::UpdateLibrary,
                        )
                    } else {
                        menu::Item::Button(fl!("update-library"), None, MenuAction::UpdateLibrary)
                    },
                    menu::Item::Divider,
                    menu::Item::Button(fl!("quit"), None, MenuAction::Quit),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("playlist")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("new-playlist-menu"), None, MenuAction::NewPlaylist),
                    if !selected_playlist.is_library() {
                        menu::Item::Button(
                            fl!("rename-playlist-menu"),
                            None,
                            MenuAction::RenamePlaylist,
                        )
                    } else {
                        menu::Item::ButtonDisabled(
                            fl!("rename-playlist-menu"),
                            None,
                            MenuAction::RenamePlaylist,
                        )
                    },
                    if !selected_playlist.is_library() {
                        menu::Item::Button(
                            fl!("delete-playlist-menu"),
                            None,
                            MenuAction::DeletePlaylist,
                        )
                    } else {
                        menu::Item::ButtonDisabled(
                            fl!("delete-playlist-menu"),
                            None,
                            MenuAction::DeletePlaylist,
                        )
                    },
                    menu::Item::Divider,
                    menu::Item::Folder(fl!("add-selected-to"), selected_playlist_list),
                    if has_playlist && !selected_playlist.is_library() {
                        menu::Item::Button(
                            fl!("remove-selected"),
                            None,
                            MenuAction::RemoveSelectedFromPlaylist,
                        )
                    } else {
                        menu::Item::ButtonDisabled(
                            fl!("remove-selected"),
                            None,
                            MenuAction::RemoveSelectedFromPlaylist,
                        )
                    },
                    menu::Item::Divider,
                    menu::Item::Folder(fl!("add-now-playing-to"), now_playing_playlist_list),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("select-all"), None, MenuAction::SelectAll),
                    menu::Item::Divider,
                    if has_playlist {
                        menu::Item::Button(fl!("move-up"), None, MenuAction::MoveNavUp)
                    } else {
                        menu::Item::ButtonDisabled(fl!("move-up"), None, MenuAction::MoveNavUp)
                    },
                    if has_playlist {
                        menu::Item::Button(fl!("move-down"), None, MenuAction::MoveNavDown)
                    } else {
                        menu::Item::ButtonDisabled(fl!("move-down"), None, MenuAction::MoveNavDown)
                    },
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("playback")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::CheckBox(
                        fl!("shuffle"),
                        None,
                        app.state.shuffle,
                        MenuAction::ToggleShuffle,
                    ),
                    menu::Item::CheckBox(
                        fl!("repeat"),
                        None,
                        app.state.repeat,
                        MenuAction::ToggleRepeat,
                    ),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("repeat-one"),
                        None,
                        repeat_one,
                        MenuAction::ToggleRepeatMode,
                    ),
                    menu::Item::CheckBox(
                        fl!("repeat-all"),
                        None,
                        repeat_all,
                        MenuAction::ToggleRepeatMode,
                    ),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), None, MenuAction::ZoomIn),
                    menu::Item::Button(fl!("zoom-out"), None, MenuAction::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("settings-menu"), None, MenuAction::Settings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("about-ethereal-waves"), None, MenuAction::About),
                ],
            ),
        ),
    ])
    .item_width(menu::ItemWidth::Uniform(250))
    .item_height(menu::ItemHeight::Dynamic(40))
    .spacing(1.0)
    .width(Length::Fill)
    .into()
}
