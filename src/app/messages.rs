use std::path::PathBuf;

use iced::window;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Tick,
    WindowOpened(window::Id),
    WindowFocused(window::Id),
    WindowCloseRequested(window::Id),
    WindowClosed(window::Id),
    TimelineChanged { window_id: window::Id, percent: f64 },
    VolumeChanged { window_id: window::Id, percent: f32 },
    PlayPausePressed(window::Id),
    ResetPressed(window::Id),
    ShufflePressed(window::Id),
    FilePicked(Option<PathBuf>),
    PlayPauseShortcut(window::Id),
    SeekByShortcut { window_id: window::Id, offset: f64 },
    NewWindowShortcut(window::Id),
    OpenShortcut(window::Id),
    CloseWindowShortcut(window::Id),
    ZoomInShortcut(window::Id),
    ZoomOutShortcut(window::Id),
    Noop,
}
