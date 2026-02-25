use iced::widget::{button, column, container, row, slider, svg, text};
use iced::{Alignment, Element, Length, Padding, window};

use crate::app::helpers::format_time;
use crate::app::messages::Message;
use crate::app::state::{PlayerWindowState, ProteusApp};
use crate::app::styles::{
    ACCENT_TEXT, ERROR_TEXT, background_style, menu_header_style, _menu_surface_style,
    timeline_slider_style, volume_slider_style,
};
use crate::native_menu::MenuAction;

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

    let main_content = column![
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

    let mut content = column![
        main_content,
        platform_footer(),
    ];

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


fn platform_footer<'a>() -> Element<'a, Message> {
    if cfg!(target_os = "macos") {
        return container(column![])
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    }

    let content = column![
        container(
            row![
                text("Ctrl+O to open, Ctrl+N for new window")
                    .size(11)
                    .color(ACCENT_TEXT)
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding([6, 23])
        .style(menu_header_style)
    ]
    .spacing(4)
    .width(Length::Fill);

    container(content).width(Length::Fill).into()
}

fn _platform_menu<'a>(
    _state: &'a ProteusApp,
    window: &'a PlayerWindowState,
    window_id: window::Id,
) -> Element<'a, Message> {
    if cfg!(target_os = "macos") {
        return container(column![])
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    }

    let menu_button_label = if window.menu_open {
        "Menu ▴"
    } else {
        "Menu ▾"
    };

    let mut content = column![
        container(
            row![
                button(text(menu_button_label).size(12))
                    .style(button::text)
                    .padding([2, 4])
                    .on_press(Message::_ToggleWindowMenu(window_id)),
                text("Ctrl+O to open, Ctrl+N for new window")
                    .size(11)
                    .color(ACCENT_TEXT)
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding([0, 2])
        .style(menu_header_style)
    ]
    .padding(Padding {
        top: 0.0,
        right: 2.0,
        bottom: 16.0,
        left: 0.0,
    })
    .spacing(4)
    .width(Length::Fill);

    if window.menu_open {
        let menu_panel = column![
            text("File").size(11).color(ACCENT_TEXT),
            _menu_item("Open…", "Ctrl+O", window_id, MenuAction::Open),
            _menu_item("New Window", "Ctrl+N", window_id, MenuAction::NewWindow),
            text("View").size(11).color(ACCENT_TEXT),
            _menu_item("Zoom In", "Ctrl+=", window_id, MenuAction::ZoomIn),
            _menu_item("Zoom Out", "Ctrl+-", window_id, MenuAction::ZoomOut),
            text("Help").size(11).color(ACCENT_TEXT),
            _menu_item("About Proteus Player", "", window_id, MenuAction::About),
        ]
        .spacing(2)
        .width(Length::Fill);

        content = content.push(
            container(menu_panel)
                .padding(8)
                .width(Length::Fill)
                .style(_menu_surface_style),
        );
    }

    container(content).width(Length::Fill).into()
}

fn _menu_item<'a>(
    label: &'a str,
    shortcut: &'a str,
    window_id: window::Id,
    action: MenuAction,
) -> Element<'a, Message> {
    button(
        row![
            text(label).size(12).width(Length::Fill),
            text(shortcut).size(11).color(ACCENT_TEXT)
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .style(button::text)
    .padding([2, 4])
    .width(Length::Fill)
    .on_press(Message::_WindowMenuAction { window_id, action })
    .into()
}
