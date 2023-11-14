use std::sync::Mutex;

use tauri::Window;
use tauri::{api::dialog, AppHandle, Manager, State};
use crate::window::create_window;
use crate::state::Windows;
use serde::{Deserialize, Serialize};

pub fn load(handle: &AppHandle) {

    let new_window = create_window(handle);
    
    load_in_window(handle, new_window.label());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoadPayload {
    path: String,
    duration: u32,
    window_label: String
}

pub fn load_in_window(handle: &AppHandle, window_label: &str) {
    let load_dialog = dialog::FileDialogBuilder::new()
        .add_filter("Prot File", &["prot"]);
    
    let window = handle.get_window(&window_label).unwrap();

    let window_label = window_label.to_string();

    load_dialog.pick_file(move |path_buf| {
        if path_buf.is_none() {
            let num_of_windows = window.app_handle().windows().len();
            window.close().unwrap();
            if num_of_windows == 1 {
                std::process::exit(0);
            }
            return;
        }

        let state: State<Mutex<Windows>> = window.state();

        let mut players = state.lock().unwrap();
        let path = path_buf.clone().unwrap();

        players.add(window_label.to_string(), &path.to_str().unwrap().to_string());

        let player = players.get(&window_label).unwrap().lock().unwrap();

        let duration = player.get_duration();
        let title = path.file_name().unwrap().to_str().unwrap();

        window.set_title(title);

        let payload = LoadPayload {
            path: path.to_str().unwrap().to_string(),
            duration: duration as u32,
            window_label: window_label.to_string()
        };
    
        println!("File selected: {:?}", path_buf);

        window.emit_to(&window_label, "LOAD_FILE", payload)
            .expect("failed to emit event");
    });
}

# [tauri::command]
pub fn play_pause(state: State<Mutex<Windows>>, window: Window) -> String {
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
        },
        None => {
            return String::from("Player not found");
        }
    }
}


# [tauri::command]
pub fn stop(state: State<Mutex<Windows>>, window: Window) {
    let players = state.lock().unwrap();
    let player = players.get(&window.label()).unwrap().lock().unwrap();
    player.stop();
}

// # [tauri::command]
// pub fn seek(state: State<Mutex<Windows>>, window: Window, position: f64) -> String {
//     println!("Seek {:?}", window);
//     let mut response = String::from("Stopped");
//     let players = state.lock().unwrap();
//     let player = players.get(&window.label()).unwrap();
//     if player.is_playing() {
//         player.seek(position);
//         response = String::from("Seeking");
//     }
//     drop(players);
//     response
// }

// # [tauri::command]
// pub fn set_volume(state: State<Mutex<Windows>>, window: Window, volume: f64) -> String {
//     println!("Set Volume {:?}", window);
//     let mut response = String::from("Stopped");
//     let players = state.lock().unwrap();
//     let player = players.get(&window.label()).unwrap();
//     if player.is_playing() {
//         player.set_volume(volume);
//         response = String::from("Volume Set");
//     }
//     drop(players);
//     response
// }

// # [tauri::command]
// pub fn get_volume(state: State<Mutex<Windows>>, window: Window) -> f64 {
//     println!("Get Volume {:?}", window);
//     let mut response = 0.0;
//     let players = state.lock().unwrap();
//     let player = players.get(&window.label()).unwrap();
//     if player.is_playing() {
//         response = player.get_volume();
//     }
//     drop(players);
//     response
// }

# [tauri::command]
pub fn get_duration(state: State<Mutex<Windows>>, window: Window) -> u32 {
    let players = state.lock().unwrap();
    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            return player.get_duration() as u32;
        },
        None => {
            return 0;
        }
    }
}

# [tauri::command]
pub fn get_position(state: State<Mutex<Windows>>, window: Window) -> u32 {
    let players = state.lock().unwrap();
    match players.get(&window.label()) {
        Some(player) => {
            let player = player.lock().unwrap();
            return player.get_time() as u32;
        },
        None => {
            return 0;
        }
    }
}

# [tauri::command]
pub fn reset(state: State<Mutex<Windows>>, window: Window) {
    let players = state.lock().unwrap();
    let player = players.get(&window.label()).unwrap().lock().unwrap();
    player.stop();
}

# [tauri::command]
pub fn get_state(state: State<Mutex<Windows>>, window: Window) -> String {
    let players = state.lock().unwrap();

    match players.get(&window.label()) {
        Some(player) => {
            let mut player = player.lock().unwrap();
            if player.is_playing() {
                return String::from("Playing");
            } else if player.is_paused() {
                return String::from("Paused");
            } else {
                return String::from("Stopped");
            }
        },
        None => {
            return String::from("Player not found");
        }
    }
}
