use crate::app::{MenuAction, Message};
use crate::fl;
use cosmic::{
    Apply, Element,
    widget::menu::{self, key_bind::KeyBind},
};
use std::collections::HashMap;

pub fn menu_bar<'a>(
    is_updating: bool,
    key_binds: &HashMap<KeyBind, MenuAction>,
) -> Element<'a, Message> {
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
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                key_binds,
                vec![
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
