import logging
import subprocess
from time import sleep
import psk_file


class Node:
    def __init__(self, startup_args):
        self.startup_args = startup_args
        self.process = None

    def start(self):
        logging.info("Starting QMC node...")
        process = subprocess.Popen(self.startup_args)
        logging.info(f"QMC process ID: {process.pid}")
        self.process = process

    def restart(self):
        logging.info("Restarting QMC node...")
        self.terminate()
        process = subprocess.Popen(self.startup_args)
        logging.info(f"QMC process ID: {process.pid}")
        self.process = process

    def terminate(self):
        logging.info("Terminating QMC node...")
        self.process.terminate()
        self.process = None


class NodeService:
    def __init__(self, node):
        self.current_node = node


node_service = NodeService(None)
