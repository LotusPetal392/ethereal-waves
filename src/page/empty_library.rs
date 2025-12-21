use crate::app::Message;
use crate::fl;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme, widget,
};
pub fn content<'a>() -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    widget::container(widget::text(fl!("add-music")))
        .padding(space_xxs)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}
