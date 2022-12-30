import logging
from threading import Thread
from external_server import ExternalServerWrapper
from local_server import LocalServerWrapper
import sys
from node import Node, NodeService
import node


path_config = sys.argv[2]
name = sys.argv[3]

logging.basicConfig(format=f'[%(asctime)s] %(levelname)s ({name}) : %(message)s', level=logging.INFO)

logging.info("Starting test node...")
node.node_service = NodeService(Node(["python3", "runner/node_simulator.py", "--config", path_config]))
node.node_service.current_node.start()

external_server = ExternalServerWrapper()
external_thread = Thread(target=external_server.run, args=())
logging.info("Starting external server...")
external_thread.daemon = True
external_thread.start()

local_server = LocalServerWrapper()
logging.info("Starting local server...")
local_server.run()
