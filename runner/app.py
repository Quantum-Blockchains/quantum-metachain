import logging
import sys
import time
from threading import Thread

import node
import psk_file
from config import config
from external_server import external_server
from local_server import local_server
from node import Node, NodeService
from psk import fetch_from_peers

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

startup_args = sys.argv[1:]
startup_args.append("--psk-file")
startup_args.append(config['psk_file_path'])
node.node_service = NodeService(Node(startup_args))

try:
    logging.info("Starting QMC runner...")
    if not psk_file.exists():
        psk = fetch_from_peers()
        psk_file.create(psk)

    # Wait until psk file is created
    while not psk_file.exists():
        time.sleep(1)

    node.node_service.current_node.start()
    external_thread = Thread(target=external_server.run, args=(None, config["external_server_port"], False))
    logging.info("Starting external server...")
    external_thread.start()

    logging.info("Starting local server...")
    local_server.run(port=config["local_server_port"])

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC processes...")
    node.node_service.current_node.terminate()
