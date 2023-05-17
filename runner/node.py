import subprocess
import time

from common.logger import log
import sys
import common.config
from threading import Thread
import common.file
import requests


class Node:
    def __init__(self, startup_args):
        self.startup_args = startup_args
        self.process = None

    def start(self):
        log.info("Starting QMC node...")

        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
        write_node_logs_thread.start()

        checking_number_of_peers = Thread(target=check_number_peers_of_node, args=())
        checking_number_of_peers.start()

    def restart(self):
        log.info("Restarting QMC node...")
        self.terminate()
        time.sleep(10)
        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
        write_node_logs_thread.start()

        checking_number_of_peers = Thread(target=check_number_peers_of_node, args=())
        checking_number_of_peers.start()

    def terminate(self):
        log.info("Terminating QMC node...")
        self.process.terminate()
        self.process = None


class NodeTest:

    def start(self):
        log.info("Starting QMC node...")
        if common.file.psk_file_manager.exists():
            common.file.psk_file_manager.remove()

    def restart(self):
        log.info("Restarting QMC node...")
        if common.file.psk_file_manager.exists():
            common.file.psk_file_manager.remove()


class NodeService:
    def __init__(self, node):
        self.current_node = node


node_service = NodeService(None)


def write_logs_node_to_file():
    with open(common.config.config_service.config.node_logs_path, 'a') as logfile:
        logfile.write("====================================================")
        logfile.write("=================== NODE STARTED ===================")
        logfile.write("====================================================\n")
        for line in node_service.current_node.process.stdout:
            sys.stdout.write(str(line, 'utf-8'))
            logfile.write(str(line, 'utf-8'))
    node_service.current_node.process.wait()


def check_number_peers_of_node():
    while True:
        time.sleep(common.config.config_service.config.checking_number_of_peers_time)
        url = f"http://localhost:{common.config.config_service.config.node_http_rpc_port}"
        data = {"id": 1, "jsonrpc": "2.0", "method": "system_peers", "params": []}
        responce = requests.post(url, json=data)
        if responce.status_code == 200:
            text = responce.json()
            number_peers = text["result"]
            if len(number_peers) == 0:
                log.info("The node has no contact with any other node.")
                common.file.psk_sig_file_manager.remove()
                common.file.psk_file_manager.remove()
                node_service.current_node.restart()
        else:
            common.file.psk_sig_file_manager.remove()
            common.file.psk_file_manager.remove()
            node_service.current_node.restart()
