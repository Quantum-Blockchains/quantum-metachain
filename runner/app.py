import logging
import subprocess
import sys
from time import sleep

from config import settings
import psk_file

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

logging.info(f"Starting QMC node...")
startup_args = sys.argv[1:]
startup_args.append("--psk-file")
startup_args.append(settings.PSK_FILE_PATH)
node_process = subprocess.Popen(startup_args)
logging.info(f"QMC process ID: {node_process.pid}")

logging.info(f"Starting local server...")
local_server_process = subprocess.Popen(["python3", "runner/local_server.py"])

try:
    while True:
        sleep(15)
        logging.info("Checking for a new pre-shared key...")
        if psk_file.exists():
            logging.info("A new pre-shared key was found - rotating the key...")
            node_process.terminate()
            node_process = subprocess.Popen(startup_args)
            logging.info(f"QMC process ID: {node_process.pid}")

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC node...")
    node_process.terminate()
    local_server_process.terminate()
