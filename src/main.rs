mod app;
mod native_menu;
mod playback;

use std::path::PathBuf;

fn main() -> iced::Result {
    set_app_menu_name();
    app::run(parse_initial_path())
}

#[cfg(target_os = "macos")]
fn set_app_menu_name() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    use objc2_foundation::{NSProcessInfo, NSString};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let _app = NSApplication::sharedApplication(mtm);
    let process_info = NSProcessInfo::processInfo();
    let name = NSString::from_str("Proteus Player");
    process_info.setProcessName(&name);
}

#[cfg(not(target_os = "macos"))]
fn set_app_menu_name() {}

fn parse_initial_path() -> Option<PathBuf> {
    let mut args = std::env::args_os();
    let _ = args.next();

    while let Some(arg) = args.next() {
        if arg == "--open"
            && let Some(path) = args.next()
        {
            return Some(PathBuf::from(path));
        }

        if !arg.to_string_lossy().starts_with('-') {
            return Some(PathBuf::from(arg));
        }
    }

    None
}
