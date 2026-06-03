use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use crate::lcu;
use crate::platform::lcu_auth;

pub enum AcceptorCommand {
    Start,
    Pause,
    Shutdown,
}

pub fn run(receiver: Receiver<AcceptorCommand>) {
    let mut paused = false;
    let mut auth = lcu_auth::get_lcu_auth();

    loop {
        while let Ok(command) = receiver.try_recv() {
            match command {
                AcceptorCommand::Start => paused = false,
                AcceptorCommand::Pause => paused = true,
                AcceptorCommand::Shutdown => return,
            }
        }

        if paused {
            thread::sleep(Duration::from_millis(1000));
            continue;
        }

        match lcu::get_phase(&auth) {
            Ok(lcu::GameflowPhase::Matchmaking) => thread::sleep(Duration::from_millis(500)),
            Ok(lcu::GameflowPhase::ReadyCheck) => {
                let _ = lcu::accept_match(&auth);
                thread::sleep(Duration::from_millis(2000));
            }
            Ok(_) => thread::sleep(Duration::from_millis(5000)),
            Err(_) => {
                thread::sleep(Duration::from_millis(10000));
                auth = lcu_auth::get_lcu_auth();
            }
        }
    }
}
