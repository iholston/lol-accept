use std::sync::mpsc::Sender;

use crate::acceptor::AcceptorCommand;
use crate::reg;

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

    pub fn quit(&self) {
        let _ = self.acceptor_tx.send(AcceptorCommand::Shutdown);
    }

    pub fn add_to_startup(&self) {
        let _ = reg::cleanup_stale_registry();
        let _ = reg::add_to_startup();
    }

    pub fn remove_from_startup(&self) {
        let _ = reg::cleanup_stale_registry();
        let _ = reg::remove_from_startup();
    }
}
