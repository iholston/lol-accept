use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use crate::lcu;
use crate::platform::lcu_auth;

pub enum AcceptorCommand {
    Start,
    Pause,
    DodgeLobby,
    Shutdown,
}

pub fn run(receiver: Receiver<AcceptorCommand>) {
    let mut paused = false;
    let mut auth = lcu_auth::discover();

    loop {
        while let Ok(command) = receiver.try_recv() {
            match command {
                AcceptorCommand::Start => paused = false,
                AcceptorCommand::Pause => paused = true,
                AcceptorCommand::DodgeLobby => {
                    if auth.is_none() {
                        auth = lcu_auth::discover();
                    }

                    if let Some(current_auth) = &auth {
                        if lcu::dodge_lobby(current_auth).is_err() {
                            auth = None;
                        }
                    }
                }
                AcceptorCommand::Shutdown => return,
            }
        }

        if paused {
            thread::sleep(Duration::from_millis(1000));
            continue;
        }

        match auth.as_ref().map(lcu::get_phase) {
            Some(Ok(lcu::GameflowPhase::Matchmaking)) => thread::sleep(Duration::from_millis(500)),
            Some(Ok(lcu::GameflowPhase::ReadyCheck)) => {
                if let Some(current_auth) = &auth {
                    let _ = lcu::accept_match(current_auth);
                }
                thread::sleep(Duration::from_millis(2000));
            }
            Some(Ok(_)) => thread::sleep(Duration::from_millis(5000)),
            Some(Err(_)) | None => {
                thread::sleep(Duration::from_millis(10000));
                auth = lcu_auth::discover();
            }
        }
    }
}
