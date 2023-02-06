import logging
import config
import sys


log_formatter = logging.Formatter('[%(asctime)s] %(levelname)s : %(message)s')

logging.getLogger("werkzeug").setLevel("WARNING")
log = logging.getLogger()

log.setLevel(logging.INFO)
consoleHandler = logging.StreamHandler()
consoleHandler.setFormatter(log_formatter)
log.addHandler(consoleHandler)


def add_logs_andler_file():
    file_handler = logging.FileHandler(f"{config.config_service.current_config.path_logs_runner}")
    file_handler.setFormatter(log_formatter)
    log.addHandler(file_handler)


def log_format_for_test():
    log_formatter_for_test = logging.Formatter(f'[%(asctime)s] %(levelname)s ({sys.argv[3]}) : %(message)s')
    consoleHandler.setFormatter(log_formatter_for_test)
