// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod menu;
mod window;
mod file;
mod state;

use file::*;

fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, play_pause, stop, get_duration, get_position, reset, get_state, seek, refresh_tracks])
        .manage(Mutex::new(state::Windows::new()))
        .menu(menu::make_menu("Proteus - Player"))
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "load" => {
                //     tauri::WindowBuilder::new()
                //         .title("Proteus - Player")
                //         .build(tauri::generate_context!())
                //         .unwrap();
                    println!("New Window");
                    file::load(&event.window().app_handle());

                }
                "quit" => {
                    std::process::exit(0);
                }
                "close" => {
                    event.window().close().unwrap();
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let window = window::create_window(&app.handle());

    file::load_in_window(&app.handle(), &window.label());


    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
