use crate::app::Message;
use crate::fl;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme,
    widget::{Column, Row, text},
};
pub fn content<'a>() -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    let content = Column::new()
        .push(Row::new().push(text(fl!("loading"))).spacing(4))
        .padding(space_xxs)
        .width(Length::Fill)
        .align_x(Alignment::Center);

    content.into()
}
