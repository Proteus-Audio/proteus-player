use iced::widget::{button, column, container, row, slider, svg, text};
use iced::{Alignment, Element, Length, window};

use crate::app::helpers::format_time;
use crate::app::messages::Message;
use crate::app::state::{PlayerWindowState, ProteusApp};
use crate::app::styles::{
    ACCENT_TEXT, ERROR_TEXT, background_style, timeline_slider_style, volume_slider_style,
};

pub(crate) fn view(state: &ProteusApp, window_id: window::Id) -> Element<'_, Message> {
    if let Some(window) = state.windows.get(&window_id) {
        return window_view(state, window, window_id);
    }

    container(text("Loading...").color(ACCENT_TEXT))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(background_style)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn window_view<'a>(
    state: &'a ProteusApp,
    window: &'a PlayerWindowState,
    window_id: window::Id,
) -> Element<'a, Message> {
    const ROW_WIDTH: f32 = 304.0;
    const TIMELINE_SLIDER_WIDTH: f32 = 232.0;

    let timeline = row![
        text(format_time(window.current_time))
            .size(12)
            .width(Length::Fixed(30.0))
            .color(ACCENT_TEXT),
        slider(0.0..=100.0, window.current_time_percent, move |percent| {
            Message::TimelineChanged { window_id, percent }
        })
        .step(0.1)
        .width(Length::Fixed(TIMELINE_SLIDER_WIDTH))
        .style(timeline_slider_style),
        text(format_time(window.duration.unwrap_or(0.0)))
            .size(12)
            .width(Length::Fixed(30.0))
            .color(ACCENT_TEXT),
    ]
    .align_y(Alignment::Center)
    .spacing(6)
    .width(Length::Fixed(ROW_WIDTH));

    let play_icon = if window.playing {
        state.icons.pause.clone()
    } else {
        state.icons.play.clone()
    };

    let controls = row![
        container(
            button(svg(state.icons.reset.clone()).width(15).height(15))
                .style(button::text)
                .padding(0)
                .on_press(Message::ResetPressed(window_id)),
        )
        .width(Length::Fill)
        .align_x(Alignment::End),
        container(
            button(svg(play_icon).width(30).height(30))
                .style(button::text)
                .padding(0)
                .on_press(Message::PlayPausePressed(window_id)),
        )
        .center_x(Length::Fill)
        .width(Length::Fixed(130.0)),
        container(
            button(svg(state.icons.shuffle.clone()).width(15).height(15))
                .style(button::text)
                .padding(0)
                .on_press(Message::ShufflePressed(window_id)),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fixed(ROW_WIDTH))
    .spacing(0);

    let volume = row![
        svg(state.icons.volume_icon(window.volume_percent))
            .width(16)
            .height(16),
        slider(0.0..=100.0, window.volume_percent, move |percent| {
            Message::VolumeChanged { window_id, percent }
        })
        .step(1.0)
        .width(Length::Fixed(ROW_WIDTH - 22.0))
        .style(volume_slider_style),
    ]
    .align_y(Alignment::Center)
    .spacing(6)
    .width(Length::Fixed(ROW_WIDTH));

    let mut content = column![
        container(timeline)
            .width(Length::Fill)
            .center_x(Length::Fill),
        container(controls)
            .width(Length::Fill)
            .center_x(Length::Fill),
        container(volume).width(Length::Fill).center_x(Length::Fill)
    ]
    .align_x(Alignment::Center)
    .spacing(6)
    .padding(12)
    .width(Length::Fill)
    .height(Length::Fill);

    if let Some(error) = &window.last_error {
        content = content.push(text(error.clone()).size(11).color(ERROR_TEXT));
    } else if let Some(error) = &state.global_error {
        content = content.push(text(error.clone()).size(11).color(ERROR_TEXT));
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(background_style)
        .into()
}
