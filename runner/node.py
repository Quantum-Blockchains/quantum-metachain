import subprocess
from utils import log
import sys
import config
from threading import Thread


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

    def restart(self):
        log.info("Restarting QMC node...")
        self.terminate()
        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
        write_node_logs_thread.start()

    def terminate(self):
        log.info("Terminating QMC node...")
        self.process.terminate()
        self.process = None


class NodeService:
    def __init__(self, node):
        self.current_node = node


node_service = NodeService(None)


def write_logs_node_to_file():
    with open(config.config_service.current_config.path_logs_node, 'w') as logfile:
        for line in node_service.current_node.process.stdout:
            sys.stdout.write(str(line, 'utf-8'))
            logfile.write(str(line, 'utf-8'))
    node_service.current_node.process.wait()
