use crate::app::Message;
use crate::fl;
use crate::library::Library;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme,
    widget::{self, Column, Row},
};

pub fn content(library: &Library) -> Element<'_, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    let mut content = Column::new();

    // Header row
    content = content.push(
        Row::new()
            .spacing(space_xxs)
            .push(
                widget::text::heading("#")
                    .align_x(Alignment::End)
                    .width(Length::Fixed(40.0)),
            )
            .push(widget::text::heading(fl!("title")).width(Length::FillPortion(1)))
            .push(widget::text::heading(fl!("album")).width(Length::FillPortion(1)))
            .push(widget::text::heading(fl!("artist")).width(Length::FillPortion(1))),
    );
    content = content.push(widget::divider::horizontal::light());

    // Row data for each file
    let mut rows = Column::new();
    let total = library.media.len();

    for (i, metadata) in library.media.values().enumerate() {
        let id = metadata.id.clone().unwrap();
        let row = widget::mouse_area(
            Row::new()
                .spacing(space_xxs)
                .height(Length::Fixed(20.0))
                .push(
                    widget::text(format!("{}", i + 1))
                        .width(Length::Fixed(40.0))
                        .align_x(Alignment::End),
                )
                .push(
                    widget::text(metadata.title.as_deref().unwrap_or(""))
                        .width(Length::FillPortion(1)),
                )
                .push(
                    widget::text(metadata.album.as_deref().unwrap_or(""))
                        .width(Length::FillPortion(1)),
                )
                .push(
                    widget::text(metadata.artist.as_deref().unwrap_or(""))
                        .width(Length::FillPortion(1)),
                ),
        )
        .on_double_click(Message::ChangeTrack(id));

        rows = rows.push(row);

        if i + 1 < total {
            rows = rows.push(widget::divider::horizontal::light());
        }
    }

    let scroller = widget::scrollable(rows).on_scroll(|viewport| Message::ListViewScroll(viewport));

    content = content.push(scroller);

    content.into()
}
