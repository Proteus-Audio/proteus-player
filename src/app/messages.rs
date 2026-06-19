use std::path::PathBuf;

use iced::window;

use crate::native_menu::MenuAction;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Tick,
    WindowOpened(window::Id),
    WindowFocused(window::Id),
    WindowCloseRequested(window::Id),
    WindowClosed(window::Id),
    TimelineChanged {
        window_id: window::Id,
        percent: f64,
    },
    VolumeChanged {
        window_id: window::Id,
        percent: f32,
    },
    PlayPausePressed(window::Id),
    ResetPressed(window::Id),
    ShufflePressed(window::Id),
    #[cfg(not(target_os = "macos"))]
    FilePicked {
        generation: u64,
        path: Option<PathBuf>,
    },
    #[cfg(target_os = "macos")]
    MacOpenDialogFinished {
        generation: u64,
        accepted: bool,
    },
    RecentFilesLoaded(Result<Vec<PathBuf>, String>),
    RecentFilesValidated {
        generation: u64,
        files: Vec<PathBuf>,
    },
    RecentFilesPersisted {
        generation: u64,
        result: Result<(), String>,
    },
    PlayPauseShortcut(window::Id),
    SeekByShortcut {
        window_id: window::Id,
        offset: f64,
    },
    NewWindowShortcut(window::Id),
    OpenShortcut(window::Id),
    CloseWindowShortcut(window::Id),
    ZoomInShortcut(window::Id),
    ZoomOutShortcut(window::Id),
    _ToggleWindowMenu(window::Id),
    _WindowMenuAction {
        window_id: window::Id,
        action: MenuAction,
    },
    Noop,
}
