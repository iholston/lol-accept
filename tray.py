import pystray
from PIL import Image


class TrayIcon:

    def __init__(self, acceptor):
        self.name = "Heimerdinger"
        self.display_name = "Heimerdinger"
        self.image = Image.open("icon.png")
        self.start_pause = "Pause"
        self.acceptor = acceptor
        self.start_pause_item = pystray.MenuItem(lambda text: self.start_pause, self.pause)
        self.exit_item = pystray.MenuItem("Exit", self.exit)
        self.menu = pystray.Menu(self.start_pause_item, self.exit_item)
        self.icon = pystray.Icon(self.name, self.image, self.display_name, self.menu)

    def run(self):
        self.acceptor.run()
        self.icon.run()

    def pause(self):
        if self.acceptor.paused:
            self.start_pause = "Pause"
            self.icon.update_menu()
            self.acceptor.paused = False
        else:
            self.start_pause = "Start"
            self.icon.update_menu()
            self.acceptor.paused = True

    def exit(self):
        self.acceptor.terminate = True
        self.icon.stop()
