use crate::Acceptor;
use std::thread;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;
use std::sync::atomic::Ordering;
use winit::event_loop::{ControlFlow, EventLoopBuilder};

const ICON_BYTES: &'static [u8] = include_bytes!("../resources/icon.ico");

pub struct TrayApp {
    acceptor: Acceptor,
}

impl TrayApp {
    pub fn new(acceptor: Acceptor) -> Self {
        TrayApp { acceptor }
    }

    pub fn run(&mut self) {
        let tray_menu = Menu::new();
        let menu_start = MenuItem::new("Start", false, None);
        let menu_pause = MenuItem::new("Pause", true, None);
        let menu_quit = MenuItem::new("Quit", true, None);
        let _ = tray_menu.append_items(&[
            &menu_start,
            &PredefinedMenuItem::separator(),
            &menu_pause,
            &PredefinedMenuItem::separator(),
            &menu_quit,
        ]);
        let menu_channel = MenuEvent::receiver();

        let icon = Self::load_icon();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("LoL-Acceptor")
            .with_icon(icon)
            .build()
            .unwrap();

        let mut acceptor_clone = self.acceptor.clone();
        thread::spawn(move || {
            acceptor_clone.run();
        });
        let event_loop = EventLoopBuilder::new().build().unwrap();
        let _ = event_loop.run(move |_event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Wait);
            if let Ok(event) = menu_channel.try_recv() {
                if event.id() == menu_start.id() {
                    self.acceptor.paused.store(false, Ordering::SeqCst);
                    menu_start.set_enabled(false);
                    menu_pause.set_enabled(true);
                } else if event.id() == menu_pause.id() {
                    self.acceptor.paused.store(true, Ordering::SeqCst);
                    menu_start.set_enabled(true);
                    menu_pause.set_enabled(false);
                } else if event.id() == menu_quit.id() {
                    self.acceptor.terminate.store(true, Ordering::SeqCst);
                    event_loop.exit();
                }
            }
        });
    }

    fn load_icon() -> tray_icon::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory_with_format(&ICON_BYTES, image::ImageFormat::Ico)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
    }
}
