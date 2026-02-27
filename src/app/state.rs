use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::task::Task;
use iced::window;

use crate::app::effects::{
    open_player_window, request_open_dialog, set_macos_app_icon_from_bytes, show_about_dialog,
};
use crate::app::icons::IconSet;
use crate::app::memory::MemorySampler;
use crate::app::messages::Message;
use crate::native_menu::{MenuAction, NativeMenu};
use crate::playback::PlaybackController;

#[derive(Debug, Clone, Copy)]
enum FilePickTarget {
    NewWindow,
    OpenCommand { window_id: window::Id },
}

pub(crate) struct PlayerWindowState {
    pub(crate) playback: PlaybackController,
    pub(crate) current_time_percent: f64,
    pub(crate) duration: Option<f64>,
    pub(crate) current_time: f64,
    pub(crate) volume_percent: f32,
    pub(crate) playing: bool,
    pub(crate) last_error: Option<String>,
    pub(crate) zoom_factor: f64,
    pub(crate) window_title: String,
    pub(crate) menu_open: bool,
    timeline_override_until: Option<Instant>,
    volume_override_until: Option<Instant>,
}

impl PlayerWindowState {
    fn new(path: Option<PathBuf>) -> Self {
        let mut window = Self {
            playback: PlaybackController::new(),
            current_time_percent: 0.0,
            duration: None,
            current_time: 0.0,
            volume_percent: 100.0,
            playing: false,
            last_error: None,
            zoom_factor: 1.0,
            window_title: "Proteus Player".to_owned(),
            menu_open: false,
            timeline_override_until: None,
            volume_override_until: None,
        };

        if let Some(path) = path {
            window.load(path);
        }

        window
    }

    fn load(&mut self, path: PathBuf) {
        match self.playback.load(&path) {
            Ok(()) => {
                self.last_error = None;
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    self.window_title = name.to_owned();
                }
            }
            Err(err) => self.last_error = Some(format!("Failed to load file: {err}")),
        }
    }

    fn refresh_status(&mut self) {
        let status = self.playback.status();
        let now = Instant::now();

        self.duration = status.duration;
        self.current_time = status.time;
        self.playing = status.playing;

        if self
            .timeline_override_until
            .is_none_or(|deadline| now >= deadline)
        {
            self.current_time_percent = status
                .duration
                .filter(|duration| *duration > 0.0)
                .map(|duration| (status.time / duration * 100.0).clamp(0.0, 100.0))
                .unwrap_or(0.0);
            self.timeline_override_until = None;
        }

        if self
            .volume_override_until
            .is_none_or(|deadline| now >= deadline)
        {
            self.volume_percent = (status.volume * 100.0).clamp(0.0, 100.0);
            self.volume_override_until = None;
        }
    }

    pub(crate) fn set_timeline_percent(&mut self, percent: f64) {
        self.current_time_percent = percent;
        self.timeline_override_until = Some(Instant::now() + Duration::from_millis(250));

        if let Some(duration) = self.duration {
            self.playback.seek(duration * percent / 100.0);
        }
    }

    pub(crate) fn set_volume_percent(&mut self, percent: f32) {
        self.volume_percent = percent;
        self.volume_override_until = Some(Instant::now() + Duration::from_millis(250));
        self.playback.set_volume(percent / 100.0);
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.playback.is_loaded()
    }

    pub(crate) fn load_path(&mut self, path: PathBuf) {
        self.load(path);
    }
}

pub(crate) struct ProteusApp {
    pub(crate) windows: HashMap<window::Id, PlayerWindowState>,
    pub(crate) focused_window: Option<window::Id>,
    pub(crate) native_menu: Option<NativeMenu>,
    native_menu_init_attempted: bool,
    app_icon_init_attempted: bool,
    pub(crate) icons: IconSet,
    pub(crate) global_error: Option<String>,
    memory_sampler: Option<MemorySampler>,
    pending_file_pick_target: FilePickTarget,
    startup_open_dialog_due_at: Option<Instant>,
}

impl ProteusApp {
    pub(crate) fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused_window: None,
            native_menu: None,
            native_menu_init_attempted: false,
            app_icon_init_attempted: false,
            icons: IconSet::new(),
            global_error: None,
            memory_sampler: MemorySampler::from_feat(),
            pending_file_pick_target: FilePickTarget::NewWindow,
            startup_open_dialog_due_at: None,
        }
    }

    pub(crate) fn open_window(&mut self, path: Option<PathBuf>) -> Task<Message> {
        let (window_id, task) = open_player_window();
        let window_state = PlayerWindowState::new(path);

        self.windows.insert(window_id, window_state);
        self.focused_window = Some(window_id);
        self.log_memory_event("window_opened");

        task.map(Message::WindowOpened)
    }

    pub(crate) fn close_window_state(&mut self, window_id: window::Id) {
        if let Some(mut window) = self.windows.remove(&window_id) {
            window.playback.shutdown();
        }

        if self.focused_window == Some(window_id) {
            self.focused_window = self.windows.keys().next().copied();
        }
        self.log_memory_event("window_closed");
    }

    pub(crate) fn ensure_native_menu(&mut self) {
        if self.native_menu_init_attempted {
            return;
        }

        self.native_menu_init_attempted = true;
        match NativeMenu::install() {
            Ok(menu) => {
                self.native_menu = Some(menu);
            }
            Err(err) => {
                self.global_error = Some(format!("Failed to install native menu: {err}"));
            }
        }
    }

    pub(crate) fn ensure_app_icon(&mut self) {
        if self.app_icon_init_attempted {
            return;
        }

        self.app_icon_init_attempted = true;
        if let Err(err) = set_macos_app_icon_from_bytes() {
            self.global_error = Some(format!("Failed to set app icon: {err}"));
        }
    }

    pub(crate) fn refresh_windows(&mut self) {
        for window in self.windows.values_mut() {
            window.refresh_status();
        }
        self.log_memory_tick();
    }

    pub(crate) fn handle_menu_action(&mut self, action: MenuAction) -> Task<Message> {
        match action {
            MenuAction::About => show_about_dialog(),
            MenuAction::NewWindow => self.start_new_window_open_dialog(),
            MenuAction::Open => self.start_open_command_dialog(),
            MenuAction::ZoomIn => {
                if let Some(window_id) = self.focused_window
                    && let Some(window) = self.windows.get_mut(&window_id)
                {
                    window.zoom_factor = (window.zoom_factor + 0.1).min(2.0);
                }
                Task::none()
            }
            MenuAction::ZoomOut => {
                if let Some(window_id) = self.focused_window
                    && let Some(window) = self.windows.get_mut(&window_id)
                {
                    window.zoom_factor = (window.zoom_factor - 0.1).max(0.5);
                }
                Task::none()
            }
        }
    }

    pub(crate) fn toggle_window_menu(&mut self, window_id: window::Id) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.menu_open = !window.menu_open;
        }
    }

    pub(crate) fn close_window_menu(&mut self, window_id: window::Id) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.menu_open = false;
        }
    }

    pub(crate) fn set_focused_window(&mut self, window_id: window::Id) {
        self.focused_window = Some(window_id);
    }

    pub(crate) fn window_mut(&mut self, window_id: window::Id) -> Option<&mut PlayerWindowState> {
        self.windows.get_mut(&window_id)
    }

    pub(crate) fn start_new_window_open_dialog(&mut self) -> Task<Message> {
        self.pending_file_pick_target = FilePickTarget::NewWindow;
        request_open_dialog()
    }

    pub(crate) fn start_open_command_dialog(&mut self) -> Task<Message> {
        self.pending_file_pick_target = self
            .focused_window
            .map(|window_id| FilePickTarget::OpenCommand { window_id })
            .unwrap_or(FilePickTarget::NewWindow);
        request_open_dialog()
    }

    pub(crate) fn handle_file_picked(&mut self, path: Option<PathBuf>) -> Task<Message> {
        let target = self.pending_file_pick_target;
        self.pending_file_pick_target = FilePickTarget::NewWindow;

        let Some(path) = path else {
            return Task::none();
        };

        match target {
            FilePickTarget::NewWindow => self.open_window(Some(path)),
            FilePickTarget::OpenCommand { window_id } => {
                if let Some(window) = self.windows.get_mut(&window_id)
                    && window.is_empty()
                {
                    window.load_path(path);
                    Task::none()
                } else {
                    self.open_window(Some(path))
                }
            }
        }
    }

    pub(crate) fn handle_external_open_path(&mut self, path: PathBuf) -> Task<Message> {
        self.startup_open_dialog_due_at = None;

        if let Some(window_id) = self.focused_window
            && let Some(window) = self.windows.get_mut(&window_id)
            && window.is_empty()
        {
            window.load_path(path);
            return Task::none();
        }

        self.open_window(Some(path))
    }

    pub(crate) fn schedule_startup_open_dialog(&mut self, delay: Duration) -> Task<Message> {
        self.startup_open_dialog_due_at = Some(Instant::now() + delay);

        Task::none()
    }

    pub(crate) fn maybe_startup_open_dialog_task(&mut self) -> Option<Task<Message>> {
        let due_at = self.startup_open_dialog_due_at?;
        if Instant::now() < due_at {
            return None;
        }

        self.startup_open_dialog_due_at = None;
        let has_loaded_window = self.windows.values().any(|window| !window.is_empty());
        if has_loaded_window {
            None
        } else {
            Some(self.start_open_command_dialog())
        }
    }

    fn log_memory_event(&mut self, reason: &str) {
        let windows = self.windows.len();
        let loaded_players = self
            .windows
            .values()
            .filter(|window| window.playback.is_loaded())
            .count();
        if let Some(sampler) = &mut self.memory_sampler {
            sampler.log_event(reason, windows, loaded_players);
        }
    }

    fn log_memory_tick(&mut self) {
        let windows = self.windows.len();
        let loaded_players = self
            .windows
            .values()
            .filter(|window| window.playback.is_loaded())
            .count();
        if let Some(sampler) = &mut self.memory_sampler {
            sampler.maybe_log_periodic(windows, loaded_players);
        }
    }
}
