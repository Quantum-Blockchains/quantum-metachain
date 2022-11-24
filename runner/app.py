import logging
import sys

import node
import psk_file
from config import settings
from local_server import local_server
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

    logging.info(f"Starting local server...")
    local_server.run(port=settings.LOCAL_SERVER_PORT)

except Exception as e:
    logging.error(str(e))
finally:
    logging.info("Closing QMC processes...")
    node.node_service.current_node.terminate()
