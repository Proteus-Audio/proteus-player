use iced::task::Task;
use iced::window;
use std::path::PathBuf;

use crate::app::messages::Message;
use crate::app::styles::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub(crate) fn request_open_dialog() -> Task<Message> {
    #[cfg(target_os = "macos")]
    {
        let path = rfd::FileDialog::new()
            .add_filter(
                "Supported Audio",
                &["prot", "mka", "wav", "mp3", "ogg", "aiff", "aif"],
            )
            .add_filter("Proteus Audio", &["prot", "mka"])
            .add_filter("Common Audio", &["wav", "mp3", "ogg", "aiff", "aif"])
            .pick_file();
        Task::done(Message::FilePicked(path))
    }

    #[cfg(not(target_os = "macos"))]
    Task::perform(
        async move {
            rfd::FileDialog::new()
                .add_filter(
                    "Supported Audio",
                    &["prot", "mka", "wav", "mp3", "ogg", "aiff", "aif"],
                )
                .add_filter("Proteus Audio", &["prot", "mka"])
                .add_filter("Common Audio", &["wav", "mp3", "ogg", "aiff", "aif"])
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
pub(crate) fn ensure_macos_open_file_handler() -> Result<(), String> {
    use std::sync::Once;

    static INSTALL_ONCE: Once = Once::new();
    static mut INSTALL_ERROR: Option<&'static str> = None;

    INSTALL_ONCE.call_once(|| {
        // SAFETY: Method patching is process-global and must be done once.
        let result = unsafe { install_open_file_methods_on_nsobject() };
        if let Err(err) = result {
            let leaked: &'static str = Box::leak(err.into_boxed_str());
            // SAFETY: Guarded by INSTALL_ONCE; no races.
            unsafe {
                INSTALL_ERROR = Some(leaked);
            }
        }
    });

    // SAFETY: Writes happen once under INSTALL_ONCE before reads.
    if let Some(err) = unsafe { INSTALL_ERROR } {
        Err(err.to_owned())
    } else {
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn ensure_macos_open_file_handler() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn take_macos_opened_files() -> Vec<PathBuf> {
    let mut guard = opened_files_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut *guard)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn take_macos_opened_files() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn opened_files_store() -> &'static std::sync::Mutex<Vec<PathBuf>> {
    use std::sync::{Mutex, OnceLock};

    static OPENED_FILES: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();
    OPENED_FILES.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(target_os = "macos")]
fn queue_opened_file(path: PathBuf) {
    let mut guard = opened_files_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    guard.push(path);
}

#[cfg(target_os = "macos")]
extern "C" fn application_open_file(
    _this: &objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _application: &objc2_app_kit::NSApplication,
    file: &objc2_foundation::NSString,
) -> objc2::runtime::Bool {
    queue_opened_file(PathBuf::from(file.to_string()));
    objc2::runtime::Bool::YES
}

#[cfg(target_os = "macos")]
extern "C" fn application_open_files(
    _this: &objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _application: &objc2_app_kit::NSApplication,
    files: &objc2_foundation::NSArray<objc2_foundation::NSString>,
) {
    for file in files {
        queue_opened_file(PathBuf::from(file.to_string()));
    }
}

#[cfg(target_os = "macos")]
unsafe fn install_open_file_methods_on_nsobject() -> Result<(), String> {
    use std::ffi::CString;

    use objc2::encode::{Encode, EncodeArguments, EncodeReturn, Encoding};
    use objc2::ffi::class_addMethod;
    use objc2::runtime::{AnyClass, AnyObject, MethodImplementation, Sel};
    use objc2::{Message, sel};

    fn count_args(sel: Sel) -> usize {
        sel.name()
            .to_bytes()
            .iter()
            .filter(|byte| **byte == b':')
            .count()
    }

    fn method_type_encoding(ret: &Encoding, args: &[Encoding]) -> Result<CString, String> {
        let mut types = format!("{}{}{}", ret, <*mut AnyObject>::ENCODING, Sel::ENCODING);
        for enc in args {
            use std::fmt::Write;
            write!(&mut types, "{enc}").map_err(|e| e.to_string())?;
        }
        CString::new(types).map_err(|e| e.to_string())
    }

    unsafe fn add_method_if_missing<T, F>(
        class: *mut AnyClass,
        sel: Sel,
        func: F,
    ) -> Result<(), String>
    where
        T: Message + ?Sized,
        F: MethodImplementation<Callee = T>,
    {
        // SAFETY: class is a valid Objective-C class object.
        let cls_ref = unsafe { &*class };
        if cls_ref.instance_method(sel).is_some() {
            return Ok(());
        }

        let encs = F::Arguments::ENCODINGS;
        let sel_args = count_args(sel);
        if sel_args != encs.len() {
            return Err(format!(
                "selector {sel:?} accepts {sel_args} arguments but function accepts {}",
                encs.len()
            ));
        }

        let types = method_type_encoding(&F::Return::ENCODING_RETURN, encs)?;
        let added = unsafe { class_addMethod(class as *mut _, sel, func.__imp(), types.as_ptr()) };

        if added.as_bool() {
            Ok(())
        } else {
            Err(format!("failed to add method {sel:?}"))
        }
    }

    let nsobject =
        AnyClass::get(c"NSObject").ok_or_else(|| "failed to get NSObject class".to_owned())?;
    let nsobject_ptr = nsobject as *const AnyClass as *mut AnyClass;

    unsafe {
        add_method_if_missing::<AnyObject, _>(
            nsobject_ptr,
            sel!(application:openFile:),
            application_open_file as unsafe extern "C" fn(_, _, _, _) -> _,
        )?;
        add_method_if_missing::<AnyObject, _>(
            nsobject_ptr,
            sel!(application:openFiles:),
            application_open_files as unsafe extern "C" fn(_, _, _, _) -> _,
        )?;
    }

    Ok(())
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
    let window_height = if cfg!(target_os = "macos") {
        WINDOW_HEIGHT
    } else {
        WINDOW_HEIGHT + 26.0
    };

    window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, window_height),
        min_size: Some(iced::Size::new(WINDOW_WIDTH, window_height)),
        max_size: Some(iced::Size::new(WINDOW_WIDTH, window_height)),
        resizable: false,
        icon: load_window_icon(),
        ..window::Settings::default()
    }
}

fn load_window_icon() -> Option<window::Icon> {
    window::icon::from_file_data(include_bytes!("../../assets/app-icon-32.png"), None).ok()
}
