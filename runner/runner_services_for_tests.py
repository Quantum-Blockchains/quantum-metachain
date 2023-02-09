from threading import Thread
from web import ExternalServerWrapper, LocalServerWrapper
import sys
from node import Node, NodeService
import node
from common.logger import log, log_format_for_test
import common.config
import common.file


path_config = sys.argv[2]
common.config.config_service = common.config.ConfigService(common.config.Config(path_config))
common.file.psk_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_file_path())
common.file.node_key_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_node_key_file_path())
common.file.psk_sig_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_sig_file_path())
name = sys.argv[3]

log_format_for_test()

log.info(f"Local port: {common.config.config_service.current_config.local_server_port}")

log.info("Starting test node...")
node.node_service = NodeService(Node(["python3", "node_simulator.py", "--config", path_config]))
node.node_service.current_node.start()

external_server = ExternalServerWrapper()
external_thread = Thread(target=external_server.run, args=())
log.info("Starting external server...")
external_thread.daemon = True
external_thread.start()

local_server = LocalServerWrapper()
log.info("Starting local server...")
local_server.run()
