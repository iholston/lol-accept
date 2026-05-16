#![windows_subsystem = "windows"]

mod acceptor;
mod app;
mod cmd;
mod lcu;
mod reg;
mod tray;

use crate::app::AppController;
use crate::tray::TrayApp;

fn main() {
    let (acceptor_tx, acceptor_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        acceptor::run(acceptor_rx);
    });

    let in_startup = reg::is_in_startup().unwrap_or(false);
    let controller = AppController::new(acceptor_tx);

    let icon = TrayApp::new(in_startup);
    icon.show(controller);
}
