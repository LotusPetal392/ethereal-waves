use crate::app::{AppModel, Message};
use crate::fl;
use crate::library::MediaMetaData;
use cosmic::{
    Element, cosmic_theme,
    iced::{
        Alignment, Font, Length,
        font::{self, Weight},
    },
    theme, widget,
};

pub fn footer<'a>(app: &AppModel) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_xs,
        space_m,
        space_l,
        ..
    } = theme::active().cosmic().spacing;

    let progress_bar_height = Length::Fixed(4.0);
    let artwork_size = 85;
    let now_playing = app.now_playing.clone().unwrap_or(MediaMetaData::new());

    // Main content container
    let mut content = widget::column().padding(space_xs);

    // Update progress area
    if app.is_updating {
        let updating_col = widget::column()
            .spacing(space_xxs)
            .push(widget::progress_bar(0.0..=100.0, app.update_percent).height(progress_bar_height))
            .push(widget::text(if app.update_progress == 0.0 {
                fl!("scanning-paths")
            } else {
                app.update_progress_display.to_string()
            }))
            .push(widget::vertical_space().height(space_xs));

        content = content.push(updating_col);
    }

    // Now playing column
    let artwork: Element<Message> = app
        .now_playing_handle
        .as_ref()
        .map(|handle| {
            widget::image(handle)
                .height(artwork_size)
                .width(artwork_size)
                .into()
        })
        .unwrap_or_else(|| {
            widget::icon(widget::icon::from_svg_bytes(include_bytes!(
                "../resources/icons/hicolor/scalable/note.svg"
            )))
            .size(artwork_size)
            .into()
        });

    let mut now_playing_text = widget::column();
    if app.now_playing.is_some() {
        now_playing_text = now_playing_text
            .push(
                widget::text(now_playing.title.unwrap_or(String::new())).font(Font {
                    weight: Weight::Bold,
                    ..Font::default()
                }),
            )
            .push(
                widget::text(now_playing.album.unwrap_or(String::new())).font(Font {
                    style: font::Style::Italic,
                    ..Font::default()
                }),
            )
            .push(widget::text(now_playing.artist.unwrap_or(String::new())))
    }

    let now_playing_column = widget::column().width(Length::FillPortion(1)).push(
        widget::row()
            .spacing(space_xxs)
            .push(artwork)
            .push(now_playing_text),
    );

    // Playback controls column
    let playback_control_column = widget::column()
        .width(Length::FillPortion(2))
        // Slider row
        .push(
            widget::row()
                .align_y(Alignment::Center)
                .spacing(space_xxs)
                .width(Length::Fill)
                .push(widget::text(app.display_playback_progress()))
                .push(
                    widget::slider(
                        0.0..=now_playing.duration.unwrap_or(0.0),
                        app.playback_progress,
                        Message::SliderSeek,
                    )
                    .on_release(Message::ReleaseSlider),
                )
                .push(widget::text(app.display_time_left())),
        )
        // Spacer above controls
        .push(widget::vertical_space().height(space_xxs))
        // Controls row
        .push(
            widget::row()
                .align_y(Alignment::Center)
                .spacing(space_xxs)
                .width(Length::Fill)
                .push(widget::horizontal_space().width(Length::Fill))
                .push(
                    widget::button::icon(widget::icon::from_name("media-skip-backward-symbolic"))
                        .on_press(Message::Previous)
                        .padding(space_xs)
                        .icon_size(space_m),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("media-playback-start-symbolic"))
                        .on_press(Message::TogglePlaying)
                        .padding(space_xs)
                        .icon_size(space_l),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("media-skip-forward-symbolic"))
                        .on_press(Message::Next)
                        .padding(space_xs)
                        .icon_size(space_m),
                )
                .push(widget::horizontal_space().width(Length::Fill)),
        );

    // Other controls column
    let other_controls_column = widget::horizontal_space().width(Length::FillPortion(1));

    let control_row = widget::row()
        .spacing(space_xxs)
        .push(now_playing_column)
        .push(playback_control_column)
        .push(other_controls_column);

    widget::layer_container(content.push(control_row))
        .layer(cosmic_theme::Layer::Primary)
        .into()
}
