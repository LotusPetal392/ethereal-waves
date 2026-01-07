use crate::app::{ContextPage, Message};
use crate::fl;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme,
    widget::{button, container, row, text},
};
pub fn content<'a>() -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    container(
        row::with_children(vec![
            text(fl!("go-to-view")).into(),
            button::link(fl!("settings"))
                .on_press(Message::ToggleContextPage(ContextPage::Settings))
                .padding(0)
                .into(),
            text(fl!("then-update-library")).into(),
        ])
        .spacing(4),
    )
    .padding(space_xxs)
    .width(Length::Fill)
    .align_x(Alignment::Center)
    .into()
}
