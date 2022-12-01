import logging
import sys
import subprocess

import node
import psk_file
from threading import Thread
from config import settings
from local_server import local_server
from external_server import external_server
from node import Node, NodeService

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

startup_args = sys.argv[1:]
startup_args.append("--psk-file")
startup_args.append(settings.PSK_FILE_PATH)
node.node_service = NodeService(Node(startup_args))


try:
    if psk_file.exists():
        node.node_service.current_node.start()
    else:
        # TODO call other nodes for psk in a loop
        pass

    external_thread = Thread(target=external_server.run, args=(None, settings.EXTERNAL_SERVER_PORT, False))
    logging.info("Starting external server...")
    external_thread.start()

    logging.info(f"Starting local server...")
    local_server.run(port=settings.LOCAL_SERVER_PORT)

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC processes...")
    node.node_service.current_node.terminate()

