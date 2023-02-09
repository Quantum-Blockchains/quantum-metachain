import sys
from threading import Thread

import node
from common.config import config, create_node_info_dir
from node import Node, NodeService, write_logs_node_to_file
from common.logger import log, add_logs_handler_file
from common.file import psk_file_manager, psk_sig_file_manager
from core import pre_shared_key
from web import ExternalServerWrapper, LocalServerWrapper

startup_args = sys.argv[1:]

create_node_info_dir()
add_logs_handler_file()
startup_args.append("--psk-file")
startup_args.append(config.config['psk_file_path'])
startup_args.append("--runner-port")
startup_args.append(str(config.config['local_server_port']))
startup_args.append("--node-key-file")
startup_args.append(config.config['node_key_file_path'])
node.node_service = NodeService(Node(startup_args[2:]))

try:
    log.info("Starting QMC runner...")
    if not psk_file_manager.exists():
        psk, signature = pre_shared_key.get_psk_from_peers()
        psk_file_manager.create(psk)
        psk_sig_file_manager.create(signature)

    node.node_service.current_node.start()
    write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
    write_node_logs_thread.start()

    external_server = ExternalServerWrapper()
    external_thread = Thread(target=external_server.run, args=())
    log.info("Starting external server...")
    external_thread.start()

    log.info("Starting local server...")
    local_server = LocalServerWrapper()
    local_server.run()

except Exception as e:
    log.error(str(e))
finally:
    log.info("Closing QMC processes...")
    node.node_service.current_node.terminate()
