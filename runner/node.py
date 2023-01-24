import subprocess
from utils import log
import sys
from config import config
from threading import Thread


class Node:
    def __init__(self, startup_args):
        self.startup_args = startup_args
        self.process = None

    def start(self):
        log.info("Starting QMC node...")

        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        # process = subprocess.Popen(self.startup_args)
        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        if sys.argv[1] != 'test' and sys.argv[0] != "runner/runner_services_for_tests.py" and sys.argv[0] != "runner/node_simulator.py":
            write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
            write_node_logs_thread.start()

    def restart(self):
        log.info("Restarting QMC node...")
        self.terminate()
        # process = subprocess.Popen(self.startup_args)

        process = subprocess.Popen(self.startup_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

        log.info(f"QMC process ID: {process.pid}")
        self.process = process
        if sys.argv[1] != 'test' and sys.argv[0] != "runner/runner_services_for_tests.py" and sys.argv[0] != "runner/node_simulator.py":
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
    logfile = open(config.config['path_logs_node'], 'w')
    for line in node_service.current_node.process.stdout:
        sys.stdout.write(str(line, 'utf-8'))
        logfile.write(str(line, 'utf-8'))
    node_service.current_node.process.wait()
