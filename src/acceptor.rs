use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use crate::cmd;
use crate::lcu;

pub enum AcceptorCommand {
    Start,
    Pause,
    Shutdown,
}

pub fn run(receiver: Receiver<AcceptorCommand>) {
    let mut paused = false;
    let mut auth = cmd::get_lcu_auth();

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
            lcu::GameflowPhase::Lobby => {
                thread::sleep(Duration::from_millis(1000));
                continue;
            }
            lcu::GameflowPhase::Matchmaking => {
                thread::sleep(Duration::from_millis(300));
                continue;
            }
            lcu::GameflowPhase::ReadyCheck => {
                lcu::accept_match(&auth);
                thread::sleep(Duration::from_millis(1000));
                continue;
            }
            _ => {}
        }

        auth = cmd::get_lcu_auth();
        thread::sleep(Duration::from_millis(5000));
    }
}
