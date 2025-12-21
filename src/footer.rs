use crate::app::Message;
use crate::fl;
use cosmic::{
    Theme, cosmic_theme,
    iced::{Alignment, Length},
    theme, widget,
};

pub fn footer<'a>(
    is_updating: bool,
    playback_progress: f32,
    update_progress: f32,
    update_total: f32,
    update_percent: f32,
    volume: f32,
) -> cosmic::widget::Container<'a, Message, Theme> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_xs,
        space_m,
        space_l,
        ..
    } = theme::active().cosmic().spacing;

    let progress_bar_height = Length::Fixed(4.0);
    let progress_bar =
        widget::progress_bar(0.0..=100.0, update_percent).height(progress_bar_height);
    let progress_count_display = format!(
        "{}/{} ({:.0}%)",
        update_progress, update_total, update_percent
    );
    let updating_label = fl!("updating-library");

    widget::container(widget::column::with_children(vec![
        // Footer
        widget::layer_container(widget::column::with_children(vec![
            // Update Row
            if is_updating {
                widget::column::with_children(vec![
                    widget::row::with_children(vec![progress_bar.into()]).into(),
                    widget::row::with_children(vec![
                        widget::text(format!("{updating_label} {progress_count_display}..."))
                            .into(),
                    ])
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
                        widget::icon(widget::icon::from_svg_bytes(include_bytes!(
                            "../resources/icons/hicolor/scalable/note.svg"
                        )))
                        .width(Length::Fixed(64.0))
                        .height(Length::Fixed(64.0))
                        .into(),
                        widget::column::with_children(vec![
                            widget::text("Title").into(),
                            widget::text("Album").into(),
                            widget::text("Artist").into(),
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
                        widget::text(String::from("0:00")).into(),
                        widget::slider(0.0..=1000.0, playback_progress, Message::TransportSeek)
                            .into(),
                        widget::text(String::from("-0:00")).into(),
                    ])
                    .align_y(Alignment::Center)
                    .padding(space_xxs)
                    .spacing(space_xs)
                    .into(),
                    // Playback control row
                    widget::row::with_children(vec![
                        widget::column::with_capacity(0).width(Length::Fill).into(),
                        widget::column::with_children(vec![
                            widget::row::with_children(vec![
                                widget::button::icon(widget::icon::from_name(
                                    "media-skip-backward-symbolic",
                                ))
                                .on_press(Message::TransportPrevious)
                                .padding(space_xs)
                                .icon_size(space_m)
                                .into(),
                                widget::button::icon(widget::icon::from_name(
                                    "media-playback-start-symbolic",
                                ))
                                .on_press(Message::TransportPlay)
                                .padding(space_xs)
                                .icon_size(space_l)
                                .into(),
                                widget::button::icon(widget::icon::from_name(
                                    "media-skip-forward-symbolic",
                                ))
                                .on_press(Message::TransportNext)
                                .padding(space_xs)
                                .icon_size(space_m)
                                .into(),
                            ])
                            .align_y(Alignment::Center)
                            .into(),
                        ])
                        .into(),
                        widget::column::with_capacity(0).width(Length::Fill).into(),
                    ])
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                    .width(Length::Fill)
                    .into(),
                ])
                .width(Length::FillPortion(2))
                .into(),
                // Right column
                widget::column::with_children(vec![
                    widget::row::with_capacity(0)
                        .width(Length::FillPortion(1))
                        .into(),
                    widget::row::with_children(vec![
                        widget::slider(0.0..=100.0, volume, Message::VolumeChanged).into(),
                    ])
                    .align_y(Alignment::Center)
                    .spacing(space_xs)
                    .width(Length::FillPortion(2))
                    .into(),
                ])
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
