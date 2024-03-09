use std::sync::atomic::Ordering;
use std::thread;

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

use crate::Acceptor;

const ICON_BYTES: &'static [u8] = include_bytes!("../resources/icon.ico");

pub struct TrayApp {
    #[allow(dead_code)]
    icon: TrayIcon,
    start: MenuItem,
    pause: MenuItem,
    quit: MenuItem,
    acceptor: Acceptor,
}

impl TrayApp {
    pub fn new(acceptor: Acceptor) -> Self {
        let tray_menu = Menu::new();
        let menu_start = MenuItem::new("Start", false, None);
        let menu_pause = MenuItem::new("Pause", true, None);
        let menu_quit = MenuItem::new("Quit", true, None);
        tray_menu
            .append_items(&[
                &menu_start,
                &PredefinedMenuItem::separator(),
                &menu_pause,
                &PredefinedMenuItem::separator(),
                &menu_quit,
            ])
            .unwrap();
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("LoL-Acceptor")
            .with_icon(Self::load_icon())
            .build()
            .unwrap();
        TrayApp {
            icon: tray_icon,
            start: menu_start,
            pause: menu_pause,
            quit: menu_quit,
            acceptor,
        }
    }

    pub fn run(&mut self) {
        let mut acceptor_clone = self.acceptor.clone();
        thread::spawn(move || {
            acceptor_clone.run();
        });

        let menu_channel = MenuEvent::receiver();
        let event_loop = EventLoopBuilder::new().build().unwrap();

        event_loop
            .run(move |_event, event_loop| {
                event_loop.set_control_flow(ControlFlow::Wait);
                if let Ok(event) = menu_channel.try_recv() {
                    if event.id() == self.start.id() {
                        self.acceptor.paused.store(false, Ordering::SeqCst);
                        self.start.set_enabled(false);
                        self.pause.set_enabled(true);
                    } else if event.id() == self.pause.id() {
                        self.acceptor.paused.store(true, Ordering::SeqCst);
                        self.start.set_enabled(true);
                        self.pause.set_enabled(false);
                    } else if event.id() == self.quit.id() {
                        self.acceptor.terminate.store(true, Ordering::SeqCst);
                        event_loop.exit();
                    }
                }
            })
            .unwrap();
    }

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
}
