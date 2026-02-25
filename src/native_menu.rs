use std::collections::HashMap;

use anyhow::{Result, anyhow};
use muda::accelerator::Accelerator;
use muda::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    About,
    NewWindow,
    Open,
    ZoomIn,
    ZoomOut,
}

pub struct NativeMenu {
    _menu: Menu,
    actions: HashMap<MenuId, MenuAction>,
}

impl NativeMenu {
    pub fn install() -> Result<Self> {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let about_id = MenuId::new("about");
        let new_window_id = MenuId::new("new_window");
        let open_id = MenuId::new("open");
        let zoom_in_id = MenuId::new("zoom_in");
        let zoom_out_id = MenuId::new("zoom_out");

        let about = MenuItem::with_id(
            about_id.clone(),
            "About Proteus Player",
            true,
            None::<Accelerator>,
        );
        let app_menu = Submenu::with_items(
            "Proteus Author",
            true,
            &[
                &about,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ],
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        let file_menu = Submenu::with_items(
            "File",
            true,
            &[
                &MenuItem::with_id(
                    new_window_id.clone(),
                    "New Window",
                    true,
                    parse_accelerator("CmdOrCtrl+N"),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    open_id.clone(),
                    "Open",
                    true,
                    parse_accelerator("CmdOrCtrl+O"),
                ),
                &PredefinedMenuItem::separator(),
            ],
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(None),
                &PredefinedMenuItem::redo(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::cut(None),
                &PredefinedMenuItem::copy(None),
                &PredefinedMenuItem::paste(None),
            ],
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        let view_menu = Submenu::with_items(
            "View",
            true,
            &[
                &MenuItem::with_id(
                    zoom_in_id.clone(),
                    "Zoom In",
                    true,
                    parse_accelerator("CmdOrCtrl+="),
                ),
                &MenuItem::with_id(
                    zoom_out_id.clone(),
                    "Zoom Out",
                    true,
                    parse_accelerator("CmdOrCtrl+-"),
                ),
            ],
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        let window_menu = Submenu::with_items(
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::close_window(None),
            ],
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        menu.append_items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu])
            .map_err(|e| anyhow!(e.to_string()))?;

        #[cfg(target_os = "macos")]
        menu.init_for_nsapp();

        actions.insert(about_id, MenuAction::About);
        actions.insert(new_window_id, MenuAction::NewWindow);
        actions.insert(open_id, MenuAction::Open);
        actions.insert(zoom_in_id, MenuAction::ZoomIn);
        actions.insert(zoom_out_id, MenuAction::ZoomOut);

        Ok(Self {
            _menu: menu,
            actions,
        })
    }

    pub fn poll_action(&self) -> Option<MenuAction> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        self.actions.get(event.id()).copied()
    }
}

fn parse_accelerator(value: &str) -> Option<Accelerator> {
    value.parse().ok()
}
