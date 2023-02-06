from threading import Thread
from web import ExternalServerWrapper, LocalServerWrapper
import sys
from node import Node, NodeService
import node
from utils import log, log_format_for_test
import config
from config import ConfigService, Config


path_config = sys.argv[2]
config.config_service = ConfigService(Config(path_config))
name = sys.argv[3]

log_format_for_test()

log.info("Starting test node...")
node.node_service = NodeService(Node(["python3", "runner/node_simulator.py", "--config", path_config]))
node.node_service.current_node.start()

external_server = ExternalServerWrapper()
external_thread = Thread(target=external_server.run, args=())
log.info("Starting external server...")
external_thread.daemon = True
external_thread.start()

local_server = LocalServerWrapper()
log.info("Starting local server...")
local_server.run()
