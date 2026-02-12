use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::{file, window};

pub fn build_menu(app: &AppHandle<Wry>) -> tauri::Result<Menu<Wry>> {
    let about = MenuItem::with_id(app, "about", "About Proteus Player", true, None::<&str>)?;

    let prot_sep1 = PredefinedMenuItem::separator(app)?;
    let prot_sep2 = PredefinedMenuItem::separator(app)?;
    let prot_sep3 = PredefinedMenuItem::separator(app)?;
    let services = PredefinedMenuItem::services(app, None)?;
    let hide = PredefinedMenuItem::hide(app, None)?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let quit = PredefinedMenuItem::quit(app, None)?;

    let prot_menu = Submenu::with_id_and_items(
        app,
        "prot",
        "Proteus Author",
        true,
        &[
            &about,
            &prot_sep1,
            &services,
            &prot_sep2,
            &hide,
            &hide_others,
            &show_all,
            &prot_sep3,
            &quit,
        ],
    )?;

    let new_window = MenuItem::with_id(app, "new_window", "New Window", true, Some("CmdOrCtrl+N"))?;
    // let save = MenuItem::with_id(app, "save", "Save", true, Some("CmdOrCtrl+S"))?;
    // let save_as = MenuItem::with_id(app, "save_as", "Save As", true, Some("CmdOrCtrl+Shift+S"))?;
    let open = MenuItem::with_id(app, "open", "Open", true, Some("CmdOrCtrl+O"))?;
    // let export_prot = MenuItem::with_id(
    //     app,
    //     "export_prot",
    //     "Export Prot File",
    //     true,
    //     Some("CmdOrCtrl+Shift+E"),
    // )?;

    let file_sep1 = PredefinedMenuItem::separator(app)?;
    let file_sep2 = PredefinedMenuItem::separator(app)?;

    let file_menu = Submenu::with_id_and_items(
        app,
        "file",
        "File",
        true,
        &[
            &new_window,
            &file_sep1,
            // &save,
            // &save_as,
            &open,
            &file_sep2,
            // &export_prot,
        ],
    )?;

    let undo = PredefinedMenuItem::undo(app, None)?;
    let redo = PredefinedMenuItem::redo(app, None)?;
    let cut = PredefinedMenuItem::cut(app, None)?;
    let copy = PredefinedMenuItem::copy(app, None)?;
    let paste = PredefinedMenuItem::paste(app, None)?;
    let edit_sep = PredefinedMenuItem::separator(app)?;

    let edit_menu = Submenu::with_id_and_items(
        app,
        "edit",
        "Edit",
        true,
        &[&undo, &redo, &edit_sep, &cut, &copy, &paste],
    )?;

    let zoom_in = MenuItem::with_id(app, "zoom", "Zoom In", true, Some("CmdOrCtrl+="))?;
    let zoom_out = MenuItem::with_id(app, "zoom_out", "Zoom Out", true, Some("CmdOrCtrl+-"))?;

    let view_menu = Submenu::with_id_and_items(app, "view", "View", true, &[&zoom_in, &zoom_out])?;

    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let close_window = PredefinedMenuItem::close_window(app, None)?;
    let window_sep = PredefinedMenuItem::separator(app)?;

    let window_menu = Submenu::with_id_and_items(
        app,
        "window",
        "Window",
        true,
        &[&minimize, &window_sep, &close_window],
    )?;

    Menu::with_id_and_items(
        app,
        "main",
        &[&prot_menu, &file_menu, &edit_menu, &view_menu, &window_menu],
    )
}

pub fn build_tray_menu(app: &AppHandle<Wry>) -> tauri::Result<Menu<Wry>> {
    let open = MenuItem::with_id(app, "tray_open", "Open...", true, None::<&str>)?;
    let quit = PredefinedMenuItem::quit(app, None)?;

    Menu::with_items(app, &[&open, &quit])
}

pub fn handle_menu_event(app: &AppHandle<Wry>, event: MenuEvent) {
    match event.id().as_ref() {
        "about" => {
            let version = app.package_info().version.to_string();
            app.dialog()
                .message(format!("v{version}\n©Adam Thomas Howard 2024"))
                .title("Proteus Player")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        "new_window" => {
            file::load(app.clone());
        }
        "open" => {
            file::load(app.clone());
        }
        "tray_open" => {
            let app = app.clone();
            app.dialog()
                .file()
                .add_filter("Audio/Prot Files", &["mka", "prot"])
                .pick_file(move |file_path| {
                    let path_option = match file_path {
                        Some(path) => path,
                        None => return,
                    };

                    let path = match path_option.as_path() {
                        Some(path) => path.to_path_buf(),
                        None => return,
                    };

                    let new_window = window::create_window(&app);
                    file::load_path_in_window(&app, new_window.label(), &path);
                });
        }
        "save" => {
            app.dialog()
                .message("Save is not implemented yet.")
                .title("Save")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        "save_as" => {
            app.dialog()
                .message("Save As is not implemented yet.")
                .title("Save As")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        "export_prot" => {
            app.dialog()
                .message("Export Prot File is not implemented yet.")
                .title("Export Prot File")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        _ => {}
    }
}
