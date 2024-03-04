mod accept;
mod tray;

use accept::Acceptor;
use tray::TrayApp;

fn main() {
    let mut app = TrayApp::new(Acceptor::new());
    app.run();
}
