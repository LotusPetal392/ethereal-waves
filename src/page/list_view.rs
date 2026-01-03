use crate::app::{AppModel, Message};
use crate::fl;
use crate::library::MediaMetaData;
use cosmic::iced_core::text::Wrapping;
use cosmic::{
    Element, cosmic_theme,
    iced::{Alignment, Color, Length},
    theme,
    widget::{self, Column, Row},
};
use std::path::PathBuf;

pub fn content(app: &AppModel) -> Element<'_, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    let mut content = Column::new();

    let chars: f32 = app.library.media.len().to_string().len() as f32;
    let number_column_width: f32 = chars * 13.0;

    // Header row
    content = content.push(
        Row::new()
            .spacing(space_xxs)
            .push(
                widget::text::heading("#")
                    .align_x(Alignment::End)
                    .width(Length::Fixed(number_column_width)),
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

    let mut list_end = app.list_start + app.list_visible_row_count + 1;
    if list_end > app.library.media.len() {
        list_end = app.library.media.len();
    }

    for (_, metadata) in media.get(app.list_start..(list_end)).unwrap_or(&[]) {
        let id = metadata.id.clone().unwrap();

        let row = widget::mouse_area(
            widget::button::custom(
                Row::new()
                    .spacing(space_xxs)
                    .height(Length::Fixed(app.list_row_height))
                    .push(
                        widget::container(
                            widget::text(format!("{}", count))
                                .width(Length::Fixed(number_column_width))
                                .align_x(Alignment::End),
                        )
                        .clip(true),
                    )
                    .push(
                        widget::container(
                            widget::text(metadata.title.as_deref().unwrap_or(""))
                                .wrapping(Wrapping::None)
                                .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    )
                    .push(
                        widget::container(
                            widget::text(metadata.album.as_deref().unwrap_or(""))
                                .wrapping(Wrapping::None)
                                .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    )
                    .push(
                        widget::container(
                            widget::text(metadata.artist.as_deref().unwrap_or(""))
                                .wrapping(Wrapping::None)
                                .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    ),
            )
            .class(button_style(
                app.list_selected.contains(metadata.id.as_ref().unwrap()),
            ))
            .padding(0),
        )
        .on_release(Message::ListSelectRow(id.clone()))
        .on_double_click(Message::ChangeTrack(id));

        rows = rows.push(row);
        if count < app.library.media.len() as u32 {
            rows = rows.push(widget::divider::horizontal::default());
        }

        count = count + 1;
    }

    let viewport_height = (app.list_row_height + 1.0) * app.library.media.len() as f32 - 1.0;

    // Vertical shim on left side the height of rows + horizontal rules
    let scrollable_contents = Row::new()
        .push(widget::vertical_space().height(Length::Fixed(viewport_height)))
        .push(widget::horizontal_space().width(space_xxs))
        .push(rows)
        .push(widget::horizontal_space().width(space_xxs));

    let scroller = widget::scrollable(scrollable_contents)
        .width(Length::Fill)
        .on_scroll(|viewport| Message::ListViewScroll(viewport));

    content = content.push(scroller);

    content.into()
}

fn button_style(selected: bool) -> theme::Button {
    theme::Button::Custom {
        active: Box::new(move |_focus, theme| button_appearance(theme, selected, false)),
        disabled: Box::new(move |theme| button_appearance(theme, selected, false)),
        hovered: Box::new(move |_focus, theme| button_appearance(theme, selected, true)),
        pressed: Box::new(move |_focus, theme| button_appearance(theme, selected, false)),
    }
}

fn button_appearance(theme: &theme::Theme, selected: bool, hovered: bool) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Style::new();

    if selected {
        appearance.background = Some(Color::from(cosmic.accent_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
        appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
    } else {
        if hovered {
            appearance.background = Some(Color::from(cosmic.accent_color()).into());
        } else {
            appearance.background = Some(Color::TRANSPARENT.into());
        }
        appearance.icon_color = Some(Color::from(cosmic.on_bg_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_color()));
    }
    appearance.outline_width = 0.0;
    appearance.border_width = 0.0;
    appearance.border_radius = cosmic.radius_xs().into();

    appearance
}
