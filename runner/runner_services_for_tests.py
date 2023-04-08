from threading import Thread
from web.local_server import LocalServerWrapper
from web.external_server import ExternalServerWrapper
import sys
from node import NodeService, NodeTest
import node
from common.logger import log, log_format_for_test
import common.config
import common.file


path_config = sys.argv[2]
common.config.init_config(path_config)
common.file.initialise_file_managers()
name = sys.argv[3]

log_format_for_test()

log.info(f"Local port: {common.config.config_service.config.local_server_port}")

log.info("Starting test node...")
node.node_service = NodeService(NodeTest())
node.node_service.current_node.start()

external_server = ExternalServerWrapper()
external_thread = Thread(target=external_server.run, args=())
log.info("Starting external server...")
external_thread.daemon = True
external_thread.start()

local_server = LocalServerWrapper()
log.info("Starting local server...")
local_server.run()
