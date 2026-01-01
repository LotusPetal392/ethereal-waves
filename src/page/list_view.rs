use crate::app::{AppModel, Message};
use crate::fl;
use crate::library::MediaMetaData;
use cosmic::iced_core::text::Wrapping;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Length},
    theme,
    widget::{self, Column, Row},
};
use std::path::PathBuf;

pub fn content(app: &AppModel) -> Element<'_, Message> {
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
    content = content.push(widget::divider::horizontal::default());

    // Row data for each file
    let mut rows = Column::new();
    rows = rows.push(widget::vertical_space().height(Length::Fixed(
        app.list_start as f32 * (app.list_row_height + 1.0),
    )));

    let media: Vec<(&PathBuf, &MediaMetaData)> = app.library.media.iter().collect();
    let mut count: u32 = app.list_start as u32 + 1;

    for (_, metadata) in media
        .get(app.list_start..(app.list_start + app.list_visible_row_count))
        .unwrap_or(&[])
    {
        let id = metadata.id.clone().unwrap();
        let row = widget::mouse_area(
            Row::new()
                .spacing(space_xxs)
                .height(Length::Fixed(app.list_row_height))
                .push(
                    widget::text(format!("{}", count))
                        .width(Length::Fixed(40.0))
                        .align_x(Alignment::End),
                )
                .push(
                    widget::text(metadata.title.as_deref().unwrap_or(""))
                        .wrapping(Wrapping::None)
                        .width(Length::FillPortion(1)),
                )
                .push(
                    widget::text(metadata.album.as_deref().unwrap_or(""))
                        .wrapping(Wrapping::None)
                        .width(Length::FillPortion(1)),
                )
                .push(
                    widget::text(metadata.artist.as_deref().unwrap_or(""))
                        .wrapping(Wrapping::None)
                        .width(Length::FillPortion(1)),
                ),
        )
        .on_double_click(Message::ChangeTrack(id));

        rows = rows.push(row);
        if count < app.library.media.len() as u32 {
            rows = rows.push(widget::divider::horizontal::light());
        }

        count = count + 1;
    }

    let viewport_height = (app.list_row_height + 1.0) * app.library.media.len() as f32 - 1.0;

    let mut scrollable_contents =
        Row::new().push(widget::vertical_space().height(Length::Fixed(viewport_height)));
    scrollable_contents = scrollable_contents.push(rows);

    let scroller = widget::scrollable(scrollable_contents)
        .width(Length::Fill)
        .on_scroll(|viewport| Message::ListViewScroll(viewport));

    content = content.push(scroller);

    content.into()
}
