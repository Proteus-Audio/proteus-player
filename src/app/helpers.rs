use iced::keyboard::{Key, Modifiers, key::Named};
use iced::window;

use crate::app::messages::Message;

pub(crate) fn handle_key_press(
    window_id: window::Id,
    key: Key,
    modifiers: Modifiers,
) -> Option<Message> {
    match key {
        Key::Named(Named::Space) => Some(Message::PlayPauseShortcut(window_id)),
        Key::Named(Named::ArrowRight) => Some(Message::SeekByShortcut {
            window_id,
            offset: 5.0,
        }),
        Key::Named(Named::ArrowLeft) => Some(Message::SeekByShortcut {
            window_id,
            offset: -5.0,
        }),
        Key::Character(value) if modifiers.command() => {
            let value = value.to_lowercase();
            match value.as_str() {
                "n" => Some(Message::NewWindowShortcut(window_id)),
                "o" => Some(Message::OpenShortcut(window_id)),
                "w" => Some(Message::CloseWindowShortcut(window_id)),
                "+" | "=" => Some(Message::ZoomInShortcut(window_id)),
                "-" => Some(Message::ZoomOutShortcut(window_id)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn format_time(time: f64) -> String {
    let safe_time = time.max(0.0);
    let minutes = (safe_time / 60.0).floor() as i64;
    let seconds = safe_time.round() as i64 % 60;
    format!("{minutes:02}:{seconds:02}")
}
