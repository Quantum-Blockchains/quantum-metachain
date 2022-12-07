import logging
import sys

import node
import psk_file
from threading import Thread
from config import config
from local_server import local_server
from external_server import external_server
from node import Node, NodeService
from runner.psk import fetch_from_peers

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

startup_args = sys.argv[1:]
startup_args.append("--psk-file")
startup_args.append(config["psk_file_path"])
node.node_service = NodeService(Node(startup_args))


try:
    if psk_file.exists():
        node.node_service.current_node.start()
    else:
        psk = fetch_from_peers()
        psk_file.create(psk)

    external_thread = Thread(target=external_server.run, args=(None, config["external_server_port"], False))
    logging.info("Starting external server...")
    external_thread.start()

    logging.info(f"Starting local server...")
    local_server.run(port=config["local_server_port"])

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC processes...")
    node.node_service.current_node.terminate()

