import subprocess
import time

from common.logger import log
import sys
import common.config
from threading import Thread
import common.file


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
        time.sleep(10)
        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
        write_node_logs_thread.start()

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
