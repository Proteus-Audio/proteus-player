use tauri::window::Color;
use tauri::{AppHandle, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;

pub fn create_window(handle: &AppHandle) -> WebviewWindow {
    // Make unused label

    let id = Uuid::new_v4();
    let label = format!("window-{}", id.to_string());

    let window = WebviewWindowBuilder::new(handle, label, WebviewUrl::App("index.html".into()))
        .title("Proteus Player")
        .inner_size(350.0, 130.0)
        .resizable(false)
        .background_color(Color(31, 31, 31, 255))
        .theme(Some(Theme::Dark))
        .build()
        .unwrap();

    window
}

pub fn create_dialog_parent_window(handle: &AppHandle) -> WebviewWindow {
    let id = Uuid::new_v4();
    let label = format!("dialog-parent-{}", id);

    WebviewWindowBuilder::new(handle, label, WebviewUrl::App("index.html".into()))
        .title("Proteus Player")
        .inner_size(10.0, 10.0)
        .resizable(false)
        .background_color(Color(31, 31, 31, 255))
        .decorations(false)
        .visible(false)
        .focused(false)
        .skip_taskbar(true)
        .theme(Some(Theme::Dark))
        .build()
        .unwrap()
}
