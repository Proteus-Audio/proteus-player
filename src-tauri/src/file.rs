use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::Windows;
use crate::window::{create_dialog_parent_window, create_window};
use proteus_lib::diagnostics::reporter::Report;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::{AppHandle, State};
use tauri::{EventTarget, Manager, WebviewWindow};

#[tauri::command]
pub fn load(handle: AppHandle) {
    let dialog_parent = create_dialog_parent_window(&handle);
    let file_path = FileDialog::new()
        .set_parent(&dialog_parent)
        .add_filter("Prot File", &["prot"])
        .pick_file();

    dialog_parent.close().ok();

    let path = match to_system_path(file_path) {
        Some(path) => path,
        None => return,
    };

    let new_window = create_window(&handle);
    load_path_in_window(&handle, new_window.label(), path);
}

#[tauri::command]
pub fn close_window(window: WebviewWindow) {
    window.close().unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoadPayload {
    path: String,
    duration: u32,
    window_label: String,
}

fn to_system_path(file_path: Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    file_path
}

fn load_path_in_window(handle: &AppHandle, window_label: &str, path: std::path::PathBuf) {
    let window = handle.get_webview_window(window_label).unwrap();
    let state: State<Mutex<Windows>> = window.state();
    let mut players = state.lock().unwrap();

    players.add(
        window_label.to_string(),
        &path.to_str().unwrap().to_string(),
    );

    let mut player = players.get(window_label).unwrap().lock().unwrap();

    let window_label_clone = window_label.to_string();
    let handle_clone = handle.clone();
    let reporter = move |Report {
                             time,
                             volume,
                             duration,
                             playing,
                         }| {
        let report = serde_json::json!({
            "time": time,
            "volume": volume,
            "duration": duration,
            "playing": playing
        });

        handle_clone
            .emit_to(
                EventTarget::webview_window(&window_label_clone),
                "UPDATE_STATUS",
                report,
            )
            .expect("failed to emit event");
    };

    player.set_reporting(Arc::new(Mutex::new(reporter)), Duration::from_millis(100));

    let duration = player.get_duration();
    let title = path.file_name().unwrap().to_str().unwrap();

    window.set_title(title).unwrap();

    let payload = LoadPayload {
        path: path.to_str().unwrap().to_string(),
        duration: duration as u32,
        window_label: window_label.to_string(),
    };

    window
        .emit_to(window_label, "LOAD_FILE", payload)
        .expect("failed to emit event");
}

#[tauri::command]
pub fn play_pause(state: State<Mutex<Windows>>, window: WebviewWindow) -> String {
    let players = state.lock().unwrap();

    match players.get(&window.label()) {
        Some(player) => {
            let mut player = player.lock().unwrap();
            if player.is_playing() {
                player.pause();
                return String::from("Paused");
            } else {
                player.play();
                return String::from("Playing");
            }
        }
        None => {
            return String::from("Player not found");
        }
    }
}

#[tauri::command]
pub fn stop(state: State<Mutex<Windows>>, window: WebviewWindow) {
    let players = state.lock().unwrap();
    let mut player = players.get(&window.label()).unwrap().lock().unwrap();
    player.stop();
    player.refresh_tracks();
}

#[tauri::command]
pub fn seek(state: State<Mutex<Windows>>, window: WebviewWindow, position: f64) {
    let players = state.lock().unwrap();
    let mut player = players.get(&window.label()).unwrap().lock().unwrap();
    player.seek(position);
}

#[tauri::command]
pub fn refresh_tracks(state: State<Mutex<Windows>>, window: WebviewWindow) {
    let players = state.lock().unwrap();
    let mut player = players.get(&window.label()).unwrap().lock().unwrap();
    player.refresh_tracks();
}

#[tauri::command]
pub fn set_volume(state: State<Mutex<Windows>>, window: WebviewWindow, volume: f32) -> String {
    let players = state.lock().unwrap();
    let mut player = players.get(&window.label()).unwrap().lock().unwrap();
    player.set_volume(volume);
    "Volume set".to_string()
}

#[tauri::command]
pub fn get_volume(state: State<Mutex<Windows>>, window: WebviewWindow) -> f32 {
    let players = state.lock().unwrap();
    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            return player.get_volume();
        }
        None => {
            return 0.8;
        }
    }
}

#[tauri::command]
pub fn get_duration(state: State<Mutex<Windows>>, window: WebviewWindow) -> u32 {
    let players = state.lock().unwrap();
    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            return player.get_duration() as u32;
        }
        None => {
            return 0;
        }
    }
}

#[tauri::command]
pub fn get_position(state: State<Mutex<Windows>>, window: WebviewWindow) -> u32 {
    let players = state.lock().unwrap();
    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            return player.get_time() as u32;
        }
        None => {
            return 0;
        }
    }
}

#[tauri::command]
pub fn reset(state: State<Mutex<Windows>>, window: WebviewWindow) {
    let players = state.lock().unwrap();
    let mut player = players.get(&window.label()).unwrap().lock().unwrap();
    player.stop();
    player.refresh_tracks();
}

#[tauri::command]
pub fn get_state(state: State<Mutex<Windows>>, window: WebviewWindow) -> String {
    let players = state.lock().unwrap();

    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            if player.is_playing() {
                return String::from("Playing");
            } else if player.is_paused() {
                return String::from("Paused");
            } else {
                return String::from("Stopped");
            }
        }
        None => {
            return String::from("Player not found");
        }
    }
}
