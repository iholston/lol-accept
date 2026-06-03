#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
compile_error!("lol-accept currently supports only Windows and macOS");

mod acceptor;
mod app;
mod lcu;
mod platform;
mod tray;

use crate::app::AppController;
use crate::tray::TrayApp;

fn main() {
    let (acceptor_tx, acceptor_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        acceptor::run(acceptor_rx);
    });

    let in_startup = platform::startup::is_enabled().unwrap_or(false);
    let controller = AppController::new(acceptor_tx);

    let icon = TrayApp::new(in_startup);
    icon.show(controller);
}
