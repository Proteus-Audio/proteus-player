// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod file;
mod menu;
mod state;
mod window;

use file::*;

fn main() {
    let app = tauri::Builder::default()
        // .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let tray_menu = menu::build_tray_menu(&handle)?;
            let mut tray_builder = tauri::tray::TrayIconBuilder::with_id("tray")
                .menu(&tray_menu)
                .tooltip("Proteus Player");

            if let Some(icon) = handle.default_window_icon().cloned() {
                tray_builder = tray_builder.icon(icon);
            }

            tray_builder.build(app)?;
            Ok(())
        })
        .menu(menu::build_menu)
        .on_menu_event(menu::handle_menu_event)
        // .plugin(tauri_plugin_notification::init())
        // .plugin(tauri_plugin_http::init())
        // .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            play_pause,
            stop,
            get_duration,
            get_position,
            reset,
            get_state,
            seek,
            refresh_tracks,
            set_volume,
            get_volume,
            load,
            close_window
        ])
        .manage(Mutex::new(state::Windows::new()))
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let window = window::create_window(&app.handle());

    file::load_in_window(&app.handle(), &window.label());

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } => {
            if !has_visible_windows {
                let window = window::create_window(_app_handle);

                file::load_in_window(_app_handle, &window.label());
            }
        }
        _ => {}
    });
}
