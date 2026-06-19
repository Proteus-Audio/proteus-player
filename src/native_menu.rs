use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use muda::accelerator::Accelerator;
use muda::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};

#[derive(Debug, Clone)]
pub enum MenuAction {
    About,
    NewWindow,
    Open,
    OpenRecent(PathBuf),
    ZoomIn,
    ZoomOut,
}

pub struct NativeMenu {
    _menu: Menu,
    actions: HashMap<MenuId, MenuAction>,
    recent_menu: Submenu,
    recent_item_ids: Vec<MenuId>,
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

        let recent_menu = Submenu::new("Open Recent", false);

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
                &recent_menu,
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
            recent_menu,
            recent_item_ids: Vec::new(),
        })
    }

    pub fn set_recent_files(&mut self, files: &[PathBuf]) -> Result<()> {
        for id in self.recent_item_ids.drain(..) {
            self.actions.remove(&id);
        }
        while self.recent_menu.remove_at(0).is_some() {}

        for (index, path) in files.iter().filter(|path| path.is_file()).enumerate() {
            let id = MenuId::new(format!("open_recent_{index}"));
            let label = path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy();
            let item = MenuItem::with_id(id.clone(), label, true, None::<Accelerator>);

            self.recent_menu
                .append(&item)
                .map_err(|e| anyhow!(e.to_string()))?;
            self.actions
                .insert(id.clone(), MenuAction::OpenRecent(path.clone()));
            self.recent_item_ids.push(id);
        }

        self.recent_menu
            .set_enabled(!self.recent_item_ids.is_empty());
        Ok(())
    }

    pub fn poll_action(&self) -> Option<MenuAction> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        self.actions.get(event.id()).cloned()
    }
}

fn parse_accelerator(value: &str) -> Option<Accelerator> {
    value.parse().ok()
}
