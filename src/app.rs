use std::sync::mpsc::Sender;

use crate::acceptor::AcceptorCommand;
use crate::platform::startup;

pub struct AppController {
    acceptor_tx: Sender<AcceptorCommand>,
}

impl AppController {
    pub fn new(acceptor_tx: Sender<AcceptorCommand>) -> Self {
        Self { acceptor_tx }
    }

    pub fn pause(&self) {
        let _ = self.acceptor_tx.send(AcceptorCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.acceptor_tx.send(AcceptorCommand::Start);
    }

    pub fn dodge_lobby(&self) {
        let _ = self.acceptor_tx.send(AcceptorCommand::DodgeLobby);
    }

    pub fn quit(&self) {
        let _ = self.acceptor_tx.send(AcceptorCommand::Shutdown);
    }

    pub fn add_to_startup(&self) {
        let _ = startup::enable();
    }

    pub fn remove_from_startup(&self) {
        let _ = startup::disable();
    }
}
