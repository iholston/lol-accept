import threading
import os
from base64 import b64encode

import requests
import urllib3
import psutil


class Acceptor:

    def __init__(self):
        self.league_proc = 'LeagueClient.exe'
        self.client_username = 'riot'
        self.client_password = ''
        self.procname = ''
        self.pid = -1
        self.host = '127.0.0.1'
        self.port = ''
        self.protocol = ''
        self.headers = ''
        self.session = requests.session()
        self.paused = False
        self.terminate = False
        urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

    def run(self):
        if not self.paused:
            pid = self.get_pid()
            if pid != -1:
                try:
                    if str(pid) != self.pid:
                        self.parse_lockfile()
                    phase = self.request('get', '/lol-gameflow/v1/gameflow-phase').json()
                    if phase == 'ReadyCheck':
                        self.request('post', '/lol-matchmaking/v1/ready-check/accept')
                except:
                    pass

        if not self.terminate:
            threading.Timer(1.0, self.run).start()

    def parse_lockfile(self) -> None:
        lockfile = open(self.find_lockfile_path(), 'r')
        data = lockfile.read()
        lockfile.close()
        data = data.split(':')
        self.procname = data[0]
        self.pid = data[1]
        self.port = data[2]
        self.client_password = data[3]
        self.protocol = data[4]
        userpass = b64encode(bytes('{}:{}'.format(self.client_username, self.client_password), 'utf-8')).decode('ascii')
        self.headers = {'Authorization': 'Basic {}'.format(userpass)}

    def find_lockfile_path(self) -> str:
        path = None
        for pid in psutil.pids():
            if psutil.Process(pid).name() == self.league_proc:
                path = psutil.Process(pid).exe()
        if path is not None:
            return os.path.join(os.path.dirname(path), 'lockfile')

    def request(self, method: str, path: str):
        url = "{}://{}:{}{}".format(self.protocol, self.host, self.port, path)
        fn = getattr(self.session, method)
        r = fn(url, verify=False, headers=self.headers, timeout=3)
        return r

    def get_pid(self):
        for proc in psutil.process_iter():
            if proc.name() == self.league_proc:
                return proc.pid
        return -1
