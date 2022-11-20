import logging
import subprocess
import sys
from os import path
from time import sleep

PROJECT_DIR = path.abspath(path.dirname(__file__))
logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

logging.info(f"Starting QMC node...")
startup_args = sys.argv[1:]
node_process = subprocess.Popen(startup_args)
logging.info(f"QMC process ID: {node_process.pid}")

try:
    while True:
        sleep(15)
        logging.info("Checking for a new pre-shared key...")
        psk_file_path = startup_args[startup_args.index('--psk-file') + 1]
        if path.exists(f"{PROJECT_DIR}/{psk_file_path}"):
            logging.info("A new pre-shared key was found - rotating the key...")
            node_process.terminate()
            node_process = subprocess.Popen(startup_args)
            logging.info(f"QMC process ID: {node_process.pid}")

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC node...")
    node_process.terminate()
