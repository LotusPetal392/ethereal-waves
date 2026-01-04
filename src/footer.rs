use crate::app::{AppModel, Message};
use crate::fl;
use crate::library::MediaMetaData;
use cosmic::widget::image;
use cosmic::{
    Theme, cosmic_theme,
    font::Font,
    iced::{
        Alignment, Length,
        font::{self, Weight},
    },
    theme, widget,
};

pub fn footer<'a>(app: &AppModel) -> cosmic::widget::Container<'a, Message, Theme> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_xs,
        space_m,
        space_l,
        ..
    } = theme::active().cosmic().spacing;

    let progress_bar_height = Length::Fixed(4.0);
    let progress_bar =
        widget::progress_bar(0.0..=100.0, app.update_percent).height(progress_bar_height);
    let progress_count_display = format!(
        "{}/{} ({:.0}%)",
        app.update_progress, app.update_total, app.update_percent
    );
    let updating_label = fl!("updating-library");
    let now_playing = app.now_playing.clone().unwrap_or(MediaMetaData::new());
    let filename = match now_playing.artwork_filename {
        Some(filename) => filename,
        None => String::new(),
    };
    let duration: f32 = now_playing.duration.unwrap_or(0.0);
    let bytes: Option<&Vec<u8>> = app.get_artwork(filename);
    let artwork_size = 75;

    widget::container(widget::column::with_children(vec![
        // Footer
        widget::layer_container(widget::column::with_children(vec![
            // Update Row
            if app.is_updating {
                widget::column::with_children(vec![
                    widget::row::with_children(vec![progress_bar.into()]).into(),
                    widget::row::with_children(vec![if app.update_total == 0.0 {
                        widget::text(fl!("scanning-paths")).into()
                    } else {
                        widget::text(format!("{updating_label} {progress_count_display}...")).into()
                    }])
                    .into(),
                ])
                .padding(space_xs)
                .spacing(space_xxs)
                .into()
            } else {
                widget::column::with_capacity(0).into()
            },
            // Playback Row
            widget::row::with_children(vec![
                // Left column
                widget::column::with_children(vec![
                    widget::row::with_children(vec![
                        if bytes.is_some() {
                            widget::image(image::Handle::from_bytes(bytes.unwrap().clone()))
                                .height(artwork_size)
                                .width(artwork_size)
                                .into()
                        } else {
                            widget::icon(widget::icon::from_svg_bytes(include_bytes!(
                                "../resources/icons/hicolor/scalable/note.svg"
                            )))
                            .size(artwork_size)
                            .into()
                        },
                        widget::column::with_children(vec![
                            widget::text(now_playing.title.unwrap_or(String::new()))
                                .font(Font {
                                    weight: Weight::Bold,
                                    ..Font::default()
                                })
                                .into(),
                            widget::text(now_playing.album.unwrap_or(String::new()))
                                .font(Font {
                                    style: font::Style::Italic,
                                    ..Font::default()
                                })
                                .into(),
                            widget::text(now_playing.artist.unwrap_or(String::new())).into(),
                        ])
                        .into(),
                    ])
                    .padding(space_xs)
                    .spacing(space_xs)
                    .into(),
                ])
                .width(Length::FillPortion(1))
                .into(),
                // Center column
                widget::column::with_children(vec![
                    // Playback progress bar row
                    widget::row::with_children(vec![
                        widget::text(app.display_playback_progress()).into(),
                        widget::slider(0.0..=duration, app.playback_progress, Message::SliderSeek)
                            .on_release(Message::ReleaseSlider)
                            .into(),
                        widget::text(app.display_time_left()).into(),
                    ])
                    .align_y(Alignment::Center)
                    .padding(space_xxs)
                    .spacing(space_xs)
                    .into(),
                    // Playback control row
                    widget::row::with_children(vec![
                        widget::column::with_capacity(0).width(Length::Fill).into(),
                        widget::button::icon(widget::icon::from_name(
                            "media-skip-backward-symbolic",
                        ))
                        .on_press(Message::Previous)
                        .padding(space_xs)
                        .icon_size(space_m)
                        .into(),
                        widget::button::icon(widget::icon::from_name(
                            "media-playback-start-symbolic",
                        ))
                        .on_press(Message::TogglePlaying)
                        .padding(space_xs)
                        .icon_size(space_l)
                        .into(),
                        widget::button::icon(widget::icon::from_name(
                            "media-skip-forward-symbolic",
                        ))
                        .on_press(Message::Next)
                        .padding(space_xs)
                        .icon_size(space_m)
                        .into(),
                        widget::column::with_capacity(0).width(Length::Fill).into(),
                    ])
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                    .width(Length::Fill)
                    .into(),
                    // Padding below playback controls
                    //widget::vertical_space().height(space_xxs).into(),
                ])
                .width(Length::FillPortion(2))
                .into(),
                // Right column
                widget::column::with_children(vec![])
                    .align_x(Alignment::Center)
                    .padding(space_xs)
                    .width(Length::FillPortion(1))
                    .into(),
            ])
            .into(),
        ]))
        .layer(cosmic_theme::Layer::Primary)
        .into(),
    ]))
}
