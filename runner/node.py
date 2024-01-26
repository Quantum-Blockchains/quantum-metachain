import subprocess
import time

from common.logger import log
import sys
import common.config
from threading import Thread
import common.file
import requests
import threading
from core import pre_shared_key
from time import sleep


class Node:
    def __init__(self, startup_args):
        self.startup_args = startup_args
        self.recovery_cron = None
        self.process = None

    def start(self):
        log.info("Starting QMC node...")

        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
        write_node_logs_thread.start()

        self.stop_event = threading.Event()
        self.recovery_cron = Thread(target=validate_node_to_network_connection, args=())
        self.recovery_cron.start()

    def restart(self):
        log.info("Restarting QMC node...")
        self.terminate()
        time.sleep(10)
        self.start()

    def terminate(self):
        log.info("Terminating QMC node...")
        self.stop_event.set()
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


def validate_node_to_network_connection():
    while not node_service.current_node.stop_event.is_set():
        time.sleep(common.config.config_service.config.recovery_check_interval)
        url = f"http://localhost:{common.config.config_service.config.node_http_rpc_port}"
        data = {"id": 1, "jsonrpc": "2.0", "method": "system_peers", "params": []}
        response = requests.post(url, json=data)
        if response.status_code == 200:
            text = response.json()
            peers_count = text["result"]
            if len(peers_count) == 0:
                log.info("Restarting the node, because it lost connection to the network")
                if common.file.psk_file_manager.exists():
                    common.file.psk_file_manager.remove()
                if common.file.psk_sig_file_manager.exists():
                    common.file.psk_sig_file_manager.remove()

                psk_obj = None
                while psk_obj is None:
                    psk_obj = pre_shared_key.get_psk_from_peers()
                    sleep(10)
                common.file.psk_file_manager.create(psk_obj.psk)
                common.file.psk_sig_file_manager.create(psk_obj.signature)

                node_service.current_node.restart()
                break
        else:
            log.info("Restarting the node, because it not answering RPC methods calls")
            common.file.psk_sig_file_manager.remove()
            common.file.psk_file_manager.remove()
            node_service.current_node.restart()
