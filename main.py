from tray import TrayIcon
from accept import Acceptor


if __name__ == '__main__':
    acceptor = Acceptor()
    tray_icon = TrayIcon(acceptor)
    tray_icon.run()
