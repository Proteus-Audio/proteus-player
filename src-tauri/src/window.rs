use tauri::{AppHandle, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;

pub fn create_window(handle: &AppHandle) -> WebviewWindow {
    // Make unused label

    let id = Uuid::new_v4();
    let label = format!("window-{}", id.to_string());

    let window = WebviewWindowBuilder::new(handle, label, WebviewUrl::App("index.html".into()))
        .title("Proteus Player")
        .inner_size(350.0, 100.0)
        .resizable(false)
        .theme(Some(Theme::Dark))
        .build()
        .unwrap();

    window
}
