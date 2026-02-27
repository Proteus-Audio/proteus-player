mod effects;
mod helpers;
mod icons;
mod memory;
mod messages;
mod state;
mod styles;
mod view;

use std::path::PathBuf;
use std::time::Duration;

use iced::event;
use iced::keyboard;
use iced::task::Task;
use iced::{Subscription, Theme, daemon, time, window};

use crate::app::helpers::handle_key_press;
use crate::app::messages::Message;
use crate::app::state::ProteusApp;

pub fn install_startup_integrations() {
    let _ = effects::ensure_macos_open_file_handler();
}

pub fn run(initial_path: Option<PathBuf>) -> iced::Result {
    daemon(
        move || {
            let mut app = ProteusApp::new();
            let task = initial_boot_task(&mut app, initial_path.clone());
            (app, task)
        },
        update,
        view::view,
    )
    .subscription(subscription)
    .theme(app_theme)
    .title(app_title)
    .scale_factor(app_scale_factor)
    .run()
}

fn update(state: &mut ProteusApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            state.ensure_app_icon();
            state.ensure_native_menu();
            state.refresh_windows();

            let mut tasks = Vec::new();

            if let Err(err) = effects::ensure_macos_open_file_handler() {
                state.global_error = Some(format!("Failed to install file-open handler: {err}"));
            }

            let opened_paths = effects::take_macos_opened_files();
            for path in opened_paths {
                tasks.push(state.handle_external_open_path(path));
            }

            let mut actions = Vec::new();
            if let Some(menu) = &state.native_menu {
                while let Some(action) = menu.poll_action() {
                    actions.push(action);
                }
            }

            for action in actions {
                tasks.push(state.handle_menu_action(action));
            }

            Task::batch(tasks)
        }
        Message::WindowOpened(window_id) | Message::WindowFocused(window_id) => {
            state.set_focused_window(window_id);
            Task::none()
        }
        Message::WindowCloseRequested(window_id) => {
            state.close_window_state(window_id);
            window::close(window_id)
        }
        Message::WindowClosed(window_id) => {
            state.close_window_state(window_id);
            if state.windows.is_empty() {
                if should_exit_on_last_window_close() {
                    iced::exit()
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        Message::TimelineChanged { window_id, percent } => {
            if let Some(window) = state.window_mut(window_id) {
                window.set_timeline_percent(percent);
            }
            Task::none()
        }
        Message::VolumeChanged { window_id, percent } => {
            if let Some(window) = state.window_mut(window_id) {
                window.set_volume_percent(percent);
            }
            Task::none()
        }
        Message::PlayPausePressed(window_id) | Message::PlayPauseShortcut(window_id) => {
            if let Some(window) = state.window_mut(window_id) {
                window.playback.play_pause();
            }
            Task::none()
        }
        Message::ResetPressed(window_id) => {
            if let Some(window) = state.window_mut(window_id) {
                window.playback.reset();
            }
            Task::none()
        }
        Message::ShufflePressed(window_id) => {
            if let Some(window) = state.window_mut(window_id) {
                window.playback.shuffle();
            }
            Task::none()
        }
        Message::NewWindowShortcut(window_id) => {
            state.set_focused_window(window_id);
            state.start_new_window_open_dialog()
        }
        Message::OpenShortcut(window_id) => {
            state.set_focused_window(window_id);
            state.start_open_command_dialog()
        }
        Message::FilePicked(path) => state.handle_file_picked(path),
        Message::SeekByShortcut { window_id, offset } => {
            if let Some(window) = state.window_mut(window_id) {
                window.playback.seek_by(offset);
            }
            Task::none()
        }
        Message::ZoomInShortcut(window_id) => {
            if let Some(window) = state.window_mut(window_id) {
                window.zoom_factor = (window.zoom_factor + 0.1).min(2.0);
            }
            Task::none()
        }
        Message::ZoomOutShortcut(window_id) => {
            if let Some(window) = state.window_mut(window_id) {
                window.zoom_factor = (window.zoom_factor - 0.1).max(0.5);
            }
            Task::none()
        }
        Message::_ToggleWindowMenu(window_id) => {
            state.set_focused_window(window_id);
            state.toggle_window_menu(window_id);
            Task::none()
        }
        Message::_WindowMenuAction { window_id, action } => {
            state.set_focused_window(window_id);
            state.close_window_menu(window_id);
            state.handle_menu_action(action)
        }
        Message::CloseWindowShortcut(window_id) => {
            state.close_window_state(window_id);
            window::close(window_id)
        }
        Message::Noop => Task::none(),
    }
}

fn subscription(_state: &ProteusApp) -> Subscription<Message> {
    Subscription::batch([
        time::every(Duration::from_millis(16)).map(|_| Message::Tick),
        window::close_requests().map(Message::WindowCloseRequested),
        window::close_events().map(Message::WindowClosed),
        event::listen_with(|event, _status, window_id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                repeat,
                ..
            }) if !repeat => handle_key_press(window_id, key, modifiers),
            iced::Event::Window(window::Event::Focused) => Some(Message::WindowFocused(window_id)),
            _ => None,
        }),
    ])
}

fn app_theme(_state: &ProteusApp, _window_id: window::Id) -> Theme {
    Theme::Dark
}

fn initial_boot_task(state: &mut ProteusApp, initial_path: Option<PathBuf>) -> Task<Message> {
    match initial_path {
        Some(path) => state.open_window(Some(path)),
        None if cfg!(target_os = "macos") => {
            let opened_paths = effects::take_macos_opened_files();
            if opened_paths.is_empty() {
                state.start_open_command_dialog()
            } else {
                let mut tasks = Vec::with_capacity(opened_paths.len());
                for path in opened_paths {
                    tasks.push(state.handle_external_open_path(path));
                }
                Task::batch(tasks)
            }
        }
        None => state.open_window(None),
    }
}

fn should_exit_on_last_window_close() -> bool {
    !cfg!(target_os = "macos")
}

fn app_title(state: &ProteusApp, window_id: window::Id) -> String {
    state
        .windows
        .get(&window_id)
        .map(|window| window.window_title.clone())
        .unwrap_or_else(|| "Proteus Player".to_owned())
}

fn app_scale_factor(state: &ProteusApp, window_id: window::Id) -> f32 {
    state
        .windows
        .get(&window_id)
        .map(|window| window.zoom_factor as f32)
        .unwrap_or(1.0)
}
