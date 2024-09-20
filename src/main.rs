#![windows_subsystem = "windows"]

mod cmd;
mod lcu;
mod tray;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::tray::TrayApp;

fn main() {
    let pause = Arc::new(AtomicBool::new(false));
    let pause_clone = Arc::clone(&pause);
    let terminate = Arc::new(AtomicBool::new(false));
    let terminate_clone = Arc::clone(&terminate);

    // Start Acceptor
    let mut auth = cmd::get_commandline();
    thread::spawn(move || {
        while !terminate.load(Ordering::SeqCst) {
            if !pause.load(Ordering::SeqCst) && !auth.auth_url.is_empty() {
                match lcu::get_phase(&auth.auth_url).as_str() {
                    "Lobby" => {
                        thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                    "Matchmaking" => {
                        thread::sleep(Duration::from_millis(300));
                        continue;
                    }
                    "ReadyCheck" => {
                        lcu::accept_match(&auth.auth_url);
                        thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                    _ => {},
                }
            }
            auth = cmd::get_commandline();
            thread::sleep(Duration::from_millis(5000));
        }
    });

    // Start Tray Icon
    let mut icon = TrayApp::new();
    icon.show(pause_clone, terminate_clone);
}
