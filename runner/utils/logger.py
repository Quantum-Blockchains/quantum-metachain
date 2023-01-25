import logging
from config import config
import sys


if sys.argv[0] == "runner/runner_services_for_tests.py":
    logFormatter = logging.Formatter(f'[%(asctime)s] %(levelname)s ({sys.argv[3]}) : %(message)s')
else:
    logFormatter = logging.Formatter('[%(asctime)s] %(levelname)s : %(message)s')

logging.getLogger("werkzeug").setLevel("WARNING")
log = logging.getLogger()

log.setLevel(logging.INFO)
consoleHandler = logging.StreamHandler()
consoleHandler.setFormatter(logFormatter)
log.addHandler(consoleHandler)


def addLogsHandlerFile():
    file_handler = logging.FileHandler(f"{config.config['path_logs_runner']}")
    file_handler.setFormatter(logFormatter)
    log.addHandler(file_handler)
