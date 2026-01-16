use crate::app::{AppModel, Message, SortBy, SortDirection};
use crate::fl;
use crate::playlist::Playlist;
use cosmic::iced_core::text::Wrapping;
use cosmic::{
    cosmic_theme,
    iced::{Alignment, Color, Length},
    theme, widget,
};

pub fn content<'a>(app: &AppModel, active_playlist: &Playlist) -> widget::Column<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_xxxs,
        ..
    } = theme::active().cosmic().spacing;

    let tracks_len = active_playlist.len();

    let mut content = widget::column();

    let chars: f32 = tracks_len.to_string().len() as f32;
    let number_column_width: f32 = chars * 13.0;

    let sort_icon: String = match app.state.sort_direction {
        SortDirection::Ascending => "pan-down-symbolic".into(),
        SortDirection::Descending => "pan-up-symbolic".into(),
    };

    // Header row
    content = content.push(
        widget::row()
            .spacing(space_xxs)
            .push(widget::horizontal_space().width(space_xxxs / 2))
            .push(
                widget::text::heading("#")
                    .align_x(Alignment::End)
                    .width(Length::Fixed(number_column_width)),
            )
            .push(
                widget::button::custom({
                    let mut row = widget::row()
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                        .push(widget::text::heading(fl!("title")));

                    if app.state.sort_by == SortBy::Title {
                        row = row.push(widget::icon::from_name(sort_icon.as_str()));
                    }

                    row
                })
                .class(button_style(false, true))
                .on_press(Message::ListViewSort(SortBy::Title))
                .padding(0)
                .width(Length::FillPortion(1)),
            )
            .push(
                widget::button::custom({
                    let mut row = widget::row()
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                        .push(widget::text::heading(fl!("album")));

                    if app.state.sort_by == SortBy::Album {
                        row = row.push(widget::icon::from_name(sort_icon.as_str()));
                    }

                    row
                })
                .class(button_style(false, true))
                .on_press(Message::ListViewSort(SortBy::Album))
                .padding(0)
                .width(Length::FillPortion(1)),
            )
            .push(
                widget::button::custom({
                    let mut row = widget::row()
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                        .push(widget::text::heading(fl!("artist")));

                    if app.state.sort_by == SortBy::Artist {
                        row = row.push(widget::icon::from_name(sort_icon));
                    }

                    row
                })
                .class(button_style(false, true))
                .on_press(Message::ListViewSort(SortBy::Artist))
                .padding(0)
                .width(Length::FillPortion(1)),
            )
            .push(widget::horizontal_space().width(space_xxxs / 2)),
    );
    content = content.push(widget::divider::horizontal::default());

    // Row data for each file
    let mut rows = widget::column();
    rows = rows.push(widget::vertical_space().height(Length::Fixed(
        app.list_start as f32 * (app.list_row_height + 1.0),
    )));

    let mut count: u32 = app.list_start as u32 + 1;

    let mut list_end = app.list_start + app.list_visible_row_count + 1;
    if list_end > tracks_len {
        list_end = tracks_len;
    }

    let wrapping = if app.config.list_text_wrap {
        Wrapping::Word
    } else {
        Wrapping::None
    };

    for (path, metadata) in active_playlist
        .tracks()
        .get(app.list_start..(list_end))
        .unwrap_or(&[])
    {
        let id = metadata.id.clone().unwrap();

        let row = widget::mouse_area(
            widget::button::custom(
                widget::row()
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
                            widget::text(
                                metadata
                                    .title
                                    .clone()
                                    .unwrap_or(String::from(path.to_string_lossy())),
                            )
                            .wrapping(wrapping)
                            .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    )
                    .push(
                        widget::container(
                            widget::text(metadata.album.clone().unwrap_or("".into()))
                                .wrapping(wrapping)
                                .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    )
                    .push(
                        widget::container(
                            widget::text(metadata.artist.clone().unwrap_or("".into()))
                                .wrapping(wrapping)
                                .width(Length::FillPortion(1)),
                        )
                        .clip(true),
                    )
                    .width(Length::Fill),
            )
            .class(button_style(
                app.list_selected.contains(metadata.id.as_ref().unwrap()),
                false,
            ))
            .on_press_down(Message::ChangeTrack(id.clone()))
            .padding(0),
        )
        .on_release(Message::ListSelectRow(id.clone()));

        rows = rows.push(row);
        if count < tracks_len as u32 {
            rows = rows.push(widget::divider::horizontal::default());
        }

        count = count + 1;
    }

    let viewport_height = if tracks_len > 0 {
        (app.list_row_height + 1.0) * tracks_len as f32 - 1.0
    } else {
        0.0
    };

    // Vertical shim on left side the height of rows + horizontal rules
    let scrollable_contents = widget::row()
        .push(widget::vertical_space().height(Length::Fixed(viewport_height)))
        .push(widget::horizontal_space().width(space_xxs))
        .push(rows)
        .push(widget::horizontal_space().width(space_xxs));

    let scroller = widget::scrollable(scrollable_contents)
        .width(Length::Fill)
        .on_scroll(|viewport| Message::ListViewScroll(viewport));

    content = content.push(scroller);

    content
}

fn button_style(selected: bool, heading: bool) -> theme::Button {
    theme::Button::Custom {
        active: Box::new(move |_focus, theme| button_appearance(theme, selected, heading, false)),
        disabled: Box::new(move |theme| button_appearance(theme, selected, heading, false)),
        hovered: Box::new(move |_focus, theme| button_appearance(theme, selected, heading, true)),
        pressed: Box::new(move |_focus, theme| button_appearance(theme, selected, heading, false)),
    }
}

fn button_appearance(
    theme: &theme::Theme,
    selected: bool,
    heading: bool,
    hovered: bool,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Style::new();

    if heading {
        appearance.background = Some(Color::TRANSPARENT.into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_color()));
    } else if selected {
        appearance.background = Some(Color::from(cosmic.accent_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
        appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
    } else if hovered {
        appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_component_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_component_color()));
    } else {
        appearance.background = Some(Color::TRANSPARENT.into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_color()));
    }
    appearance.outline_width = 0.0;
    appearance.border_width = 0.0;
    appearance.border_radius = cosmic.radius_xs().into();

    appearance
}
