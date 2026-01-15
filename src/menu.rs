use crate::app::{MenuAction, Message};
use crate::fl;
use cosmic::{
    Apply, Element,
    widget::menu::{self, key_bind::KeyBind},
};
use std::collections::HashMap;

pub fn menu_bar<'a>(
    is_updating: bool,
    view_playlist: Option<u32>,
    key_binds: &HashMap<KeyBind, MenuAction>,
) -> Element<'a, Message> {
    let has_playlist = view_playlist.is_some();
    menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(
                key_binds,
                vec![
                    if is_updating {
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
                key_binds,
                vec![
                    menu::Item::Button(fl!("new-playlist-menu"), None, MenuAction::NewPlaylist),
                    if has_playlist {
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
                    if has_playlist {
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
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                key_binds,
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
    .into()
}
