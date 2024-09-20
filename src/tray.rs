use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

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
    menu_quit: MenuItem,
}

impl TrayApp {
    pub fn new() -> Self {
        let menu_start = MenuItem::new("Start", false, None);
        let menu_pause = MenuItem::new("Pause", true, None);
        let menu_quit = MenuItem::new("Quit", true, None);
        let tray_menu = Menu::new();
        tray_menu
            .append_items(&[
                &menu_start,
                &PredefinedMenuItem::separator(),
                &menu_pause,
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
                    } else if event.id() == self.menu_quit.id() {
                        terminate.store(true, Ordering::SeqCst);
                        event_loop.exit();
                    }
                }
            })
            .unwrap();
    }
}
