use iced::widget::{container, slider};
use iced::{Color, Theme};

pub(crate) const WINDOW_WIDTH: f32 = 350.0;
pub(crate) const WINDOW_HEIGHT: f32 = 110.0;
pub(crate) const WINDOW_BG: Color = Color::from_rgb(31.0 / 255.0, 31.0 / 255.0, 31.0 / 255.0);
pub(crate) const ACCENT_TEXT: Color = Color::from_rgb(158.0 / 255.0, 158.0 / 255.0, 158.0 / 255.0);
pub(crate) const ERROR_TEXT: Color = Color::from_rgb(1.0, 120.0 / 255.0, 120.0 / 255.0);

const RAIL_BG: Color = Color::from_rgb(121.0 / 255.0, 121.0 / 255.0, 121.0 / 255.0);
const RAIL_FILL: Color = Color::from_rgb(93.0 / 255.0, 93.0 / 255.0, 93.0 / 255.0);

pub(crate) fn timeline_slider_style(_theme: &Theme, status: slider::Status) -> slider::Style {
    let rail = slider::Rail {
        backgrounds: (RAIL_FILL.into(), RAIL_BG.into()),
        width: 4.0,
        border: iced::border::rounded(0),
    };

    let hidden_handle = slider::Handle {
        shape: slider::HandleShape::Circle { radius: 0.0 },
        background: Color::TRANSPARENT.into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    };

    let visible_handle = slider::Handle {
        shape: slider::HandleShape::Circle { radius: 5.0 },
        background: Color::from_rgb(196.0 / 255.0, 196.0 / 255.0, 196.0 / 255.0).into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    };

    slider::Style {
        rail,
        handle: if matches!(status, slider::Status::Hovered | slider::Status::Dragged) {
            visible_handle
        } else {
            hidden_handle
        },
    }
}

pub(crate) fn volume_slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (RAIL_FILL.into(), RAIL_BG.into()),
            width: 4.0,
            border: iced::border::rounded(0),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 5.0 },
            background: Color::from_rgb(196.0 / 255.0, 196.0 / 255.0, 196.0 / 255.0).into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub(crate) fn background_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(WINDOW_BG)
        .color(ACCENT_TEXT)
}
