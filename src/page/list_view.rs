use crate::app::Message;
use crate::fl;
use crate::library::Library;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme, widget,
};

pub fn content<'a>(library: &Library) -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    let mut content = widget::column::with_children(vec![]);
    let mut rows = widget::column::with_children(vec![]);
    let mut count = 1;

    content = content.push(
        widget::row::with_children(vec![
            widget::text::heading(fl!("number"))
                .width(Length::Fixed(40.0))
                .align_x(Alignment::End)
                .into(),
            widget::text::heading(fl!("title"))
                .width(Length::FillPortion(1))
                .into(),
            widget::text::heading(fl!("album"))
                .width(Length::FillPortion(1))
                .into(),
            widget::text::heading(fl!("artist"))
                .width(Length::FillPortion(1))
                .into(),
        ])
        .spacing(space_xxs),
    );

    content = content.push(widget::row::with_children(vec![
        widget::divider::horizontal::default().into(),
    ]));

    for (_, metadata) in library.media.clone() {
        rows = rows.push(
            widget::mouse_area(
                widget::row::with_children(vec![
                    widget::text(format!("{}.", count))
                        .width(Length::Fixed(40.0))
                        .align_x(Alignment::End)
                        .into(),
                    widget::text(metadata.title.unwrap_or(String::new()))
                        .width(Length::FillPortion(1))
                        .into(),
                    widget::text(metadata.album.unwrap_or(String::new()))
                        .width(Length::FillPortion(1))
                        .into(),
                    widget::text(metadata.artist.unwrap_or(String::new()))
                        .width(Length::FillPortion(1))
                        .into(),
                ])
                .spacing(space_xxs),
            )
            .on_double_click(Message::ChangeTrack(metadata.id.unwrap())),
        );

        if count < library.media.len() {
            rows = rows.push(widget::divider::horizontal::light());
            count = count + 1;
        }
    }

    content = content.push(widget::scrollable(rows));

    content.padding(space_xxs).into()
}
