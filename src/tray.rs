use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, Submenu, PredefinedMenuItem};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use crate::reg;

const ICON_BYTES: &'static [u8] = include_bytes!("../assets/icon.ico");

fn load_icon() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory_with_format(&ICON_BYTES, image::ImageFormat::Ico)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

pub struct TrayApp {
    #[allow(dead_code)]
    icon: TrayIcon,
    menu_start: MenuItem,
    menu_pause: MenuItem,
    submenu_startup: Submenu,
    submenu_yes: MenuItem,
    submenu_no: MenuItem,
    menu_quit: MenuItem,
}

impl TrayApp {
    pub fn new() -> Self {
        let _ = reg::cleanup_stale_registry();
        let in_startup = reg::is_in_startup().unwrap_or(false);

        let tray_menu = Menu::new();
        let menu_start = MenuItem::new("Start", false, None);
        let menu_pause = MenuItem::new("Pause", true, None);
        let submenu_startup = Submenu::new("Run on Startup", true);
        let submenu_yes = MenuItem::new("Yes", !in_startup, None);
        let submenu_no = MenuItem::new("No", in_startup, None);
        let menu_quit = MenuItem::new("Quit", true, None);

        submenu_startup
            .append_items(&[
                &submenu_yes,
                &submenu_no,
            ])
            .unwrap();

        tray_menu
            .append_items(&[
                &menu_start,
                &menu_pause,
                &PredefinedMenuItem::separator(),
                &submenu_startup,
                &PredefinedMenuItem::separator(),
                &menu_quit,
            ])
            .unwrap();

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("LoL-Accept")
            .with_icon(load_icon())
            .build()
            .unwrap();

        TrayApp {
            icon,
            menu_start,
            menu_pause,
            submenu_startup,
            submenu_yes,
            submenu_no,
            menu_quit,
        }
    }

    pub fn show(&mut self, pause: Arc<AtomicBool>, terminate: Arc<AtomicBool>) {
        let menu_channel = MenuEvent::receiver();
        let event_loop = EventLoopBuilder::new().build().unwrap();

        event_loop
            .run(move |_event, event_loop| {
                event_loop.set_control_flow(ControlFlow::Wait);
                if let Ok(event) = menu_channel.try_recv() {
                    if event.id() == self.menu_start.id() {
                        pause.store(false, Ordering::SeqCst);
                        self.menu_start.set_enabled(false);
                        self.menu_pause.set_enabled(true);
                    } else if event.id() == self.menu_pause.id() {
                        pause.store(true, Ordering::SeqCst);
                        self.menu_start.set_enabled(true);
                        self.menu_pause.set_enabled(false);
                    } else if event.id() == self.submenu_yes.id() {
                        self.submenu_yes.set_enabled(false);
                        self.submenu_no.set_enabled(true);
                        let _ = reg::cleanup_stale_registry();
                        let _ = reg::add_to_startup();
                    } else if event.id() == self.submenu_no.id() {
                        self.submenu_no.set_enabled(false);
                        self.submenu_yes.set_enabled(true);
                        let _ = reg::cleanup_stale_registry();
                        let _ = reg::remove_from_startup();
                    }else if event.id() == self.menu_quit.id() {
                        terminate.store(true, Ordering::SeqCst);
                        event_loop.exit();
                    }
                }
            })
            .unwrap();
    }
}
