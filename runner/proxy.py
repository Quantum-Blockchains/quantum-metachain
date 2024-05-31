from common.logger import log
from os import path
import subprocess
import time
import configparser
import socket


PROXY_DIR = path.abspath(path.dirname(__file__) + "/../proxy")


class Proxy:
    def __init__(self, config_path):
        if config_path is None:
            self.startup_args = None
        else:
            self.config_path = config_path
            self.startup_args = ["java", "-cp", path.join(PROXY_DIR, "multiProxy.jar"),
                                 "io.blockchains.proxy.MainMultiProxy", config_path]
        self.process = None
        self.counter_ports = 10000

    def start(self):
        log.info("Starting proxy...")

        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process

    def restart(self):
        log.info("Restarting proxy...")
        self.terminate()
        time.sleep(10)
        self.start()

    def terminate(self):
        log.info("Terminating proxy...")
        # self.stop_event.set()
        self.process.terminate()
        self.process = None

    def add_listning(self, name_node: str, name_service: str) -> str:
        config_proxy = configparser.ConfigParser()
        config_proxy.read(self.config_path)
        ports = []
        listning = config_proxy['settings']['client.proxy.listening']
        if listning != '':
            listnings = listning.split(',')
            for item in listnings:
                tmp = item.split(':')
                if tmp[1] == name_service and tmp[2] == name_node:
                    return f'http://127.0.0.1:{tmp[0]}'
                ports.append(int(tmp[0]))
        port = None
        sock = socket.socket()
        while True:
            try:
                sock.bind(('', self.counter_ports))
                if sock.getsockname()[1] in ports:
                    self.counter_ports += 1
                    continue
                port = self.counter_ports
                self.counter_ports += 1
                break
            except Exception as err:
                self.counter_ports += 1
                continue
        if listning == '':
            config_proxy['settings']['client.proxy.listening'] = listning + f'{port}:{name_service}:{name_node}'
        else:
            config_proxy['settings']['client.proxy.listening'] = listning + f', {port}:{name_service}:{name_node}'
        with open(path.join(self.config_path), 'w') as configfile:
            config_proxy.write(configfile)

        if self.process is not None:
            self.restart()

        return f'http://127.0.0.1:{port}'


class ProxyService:
    def __init__(self, proxy):
        self.current_proxy = proxy


proxy_service = ProxyService(None)
