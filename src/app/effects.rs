use iced::task::Task;
use iced::window;

use crate::app::messages::Message;
use crate::app::styles::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub(crate) fn request_open_dialog() -> Task<Message> {
    #[cfg(target_os = "macos")]
    {
        let path = rfd::FileDialog::new()
            .add_filter("Prot File", &["prot"])
            .pick_file();
        Task::done(Message::FilePicked(path))
    }

    #[cfg(not(target_os = "macos"))]
    Task::perform(
        async move {
            rfd::FileDialog::new()
                .add_filter("Prot File", &["prot"])
                .pick_file()
        },
        Message::FilePicked,
    )
}

pub(crate) fn show_about_dialog() -> Task<Message> {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    Task::perform(
        async move {
            let _shown = rfd::MessageDialog::new()
                .set_title("Proteus Player")
                .set_description(format!("v{version}\n©Adam Thomas Howard 2024"))
                .set_level(rfd::MessageLevel::Info)
                .show();
        },
        |_| Message::Noop,
    )
}

pub(crate) fn open_player_window() -> (window::Id, Task<window::Id>) {
    window::open(player_window_settings())
}

#[cfg(target_os = "macos")]
pub(crate) fn set_macos_app_icon_from_bytes() -> Result<(), String> {
    use objc2::AnyThread;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        return Err("not on main thread".to_owned());
    };

    let app = NSApplication::sharedApplication(mtm);
    let image_data = NSData::with_bytes(include_bytes!("../../assets/app-icon/128x128.png"));
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &image_data) else {
        return Err("failed to decode app icon PNG bytes".to_owned());
    };

    // SAFETY: Called on main thread with a valid NSImage object.
    unsafe { app.setApplicationIconImage(Some(&image)) };
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn set_macos_app_icon_from_bytes() -> Result<(), String> {
    Ok(())
}

fn player_window_settings() -> window::Settings {
    window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        min_size: Some(iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        max_size: Some(iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        resizable: false,
        icon: load_window_icon(),
        ..window::Settings::default()
    }
}

fn load_window_icon() -> Option<window::Icon> {
    window::icon::from_file_data(include_bytes!("../../assets/app-icon-32.png"), None).ok()
}
