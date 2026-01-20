use crate::app::{AppModel, Message, SortBy, SortDirection};
use crate::fl;
use crate::playlist::Playlist;
use crate::playlist::Track;
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

    let search = app.search_term.as_deref().unwrap_or("").to_lowercase();

    let tracks: Vec<(usize, Track)> = if app.search_term.is_some() {
        active_playlist
            .tracks()
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, t)| {
                [
                    t.metadata.title.as_deref(),
                    t.metadata.album.as_deref(),
                    t.metadata.artist.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|v| v.to_lowercase().contains(&search))
            })
            .collect::<Vec<(usize, Track)>>()
    } else {
        active_playlist
            .tracks()
            .iter()
            .cloned()
            .enumerate()
            .collect()
    };

    let mut list_start = app.list_start;

    let tracks_len = tracks.len();
    let list_end = (list_start + app.list_visible_row_count + 1).min(tracks_len);

    if list_start >= list_end {
        list_start = 0 as usize;
    }

    let take = list_end.saturating_sub(list_start);

    // Calculations for row height
    let row_stride: f32 = app.list_row_height + app.list_divider_height;

    let chars: f32 = tracks_len.to_string().len() as f32;
    let number_column_width: f32 = chars * 11.0;

    let sort_icon: String = match app.state.sort_direction {
        SortDirection::Ascending => "pan-down-symbolic".into(),
        SortDirection::Descending => "pan-up-symbolic".into(),
    };

    let align = if app.config.list_row_align_top {
        Alignment::Start
    } else {
        Alignment::Center
    };

    let mut content = widget::column();

    content = content.push(
        widget::row()
            .spacing(space_xxs)
            .push(widget::horizontal_space().width(space_xxxs))
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
                        row = row.push(widget::icon::from_name(sort_icon.as_str()));
                    }

                    row
                })
                .class(button_style(false, true))
                .on_press(Message::ListViewSort(SortBy::Artist))
                .padding(0)
                .width(Length::FillPortion(1)),
            )
            .push(widget::horizontal_space().width(space_xxs)),
    );
    content = content.push(widget::divider::horizontal::default());

    let mut count: u32 = list_start as u32 + 1;

    let wrapping = if app.config.list_text_wrap {
        Wrapping::Word
    } else {
        Wrapping::None
    };

    let mut rows = widget::column();

    rows =
        rows.push(widget::vertical_space().height(Length::Fixed(list_start as f32 * row_stride)));

    for (index, track) in tracks.iter().skip(list_start).take(take).enumerate() {
        let id = track.1.metadata.id.clone().unwrap();

        let row_element = widget::row()
            .spacing(space_xxs)
            .height(Length::Fixed(app.list_row_height))
            .push(
                widget::container(
                    widget::text(count.to_string())
                        .width(Length::Fixed(number_column_width))
                        .align_x(Alignment::End)
                        .align_y(align)
                        .height(app.list_row_height),
                )
                .clip(true),
            )
            .push(
                widget::container(
                    widget::text(
                        track
                            .1
                            .metadata
                            .title
                            .clone()
                            .unwrap_or_else(|| track.1.path.to_string_lossy().to_string()),
                    )
                    .align_y(align)
                    .height(app.list_row_height)
                    .wrapping(wrapping)
                    .width(Length::FillPortion(1)),
                )
                .clip(true),
            )
            .push(
                widget::container(
                    widget::text(track.1.metadata.album.clone().unwrap_or_default())
                        .align_y(align)
                        .height(app.list_row_height)
                        .wrapping(wrapping)
                        .width(Length::FillPortion(1)),
                )
                .clip(true),
            )
            .push(
                widget::container(
                    widget::text(track.1.metadata.artist.clone().unwrap_or_default())
                        .align_y(align)
                        .height(app.list_row_height)
                        .wrapping(wrapping)
                        .width(Length::FillPortion(1)),
                )
                .clip(true),
            )
            .width(Length::Fill);

        let row_button = widget::button::custom(row_element)
            .class(button_style(track.1.selected, false))
            .on_press_down(Message::ChangeTrack(id.clone()))
            .padding(0);

        rows =
            rows.push(widget::mouse_area(row_button).on_release(Message::ListSelectRow(track.0)));

        let visible_count = list_end.saturating_sub(list_start);
        let is_last_visible = index + 1 == visible_count;
        if !is_last_visible {
            rows = rows.push(
                widget::container(widget::divider::horizontal::default())
                    .height(Length::Fixed(app.list_divider_height))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .clip(true),
            );
        }

        count += 1;
    }

    let viewport_height = tracks_len as f32 * row_stride;

    let scrollable_contents = widget::row()
        .push(widget::vertical_space().height(Length::Fixed(viewport_height)))
        .push(widget::horizontal_space().width(space_xxs))
        .push(rows)
        .push(widget::horizontal_space().width(space_xxs));

    let scroller = widget::scrollable(scrollable_contents)
        .id(app.list_scroll_id.clone())
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
