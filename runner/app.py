from threading import Thread

import node
import common.config
import common.file
import params
from common.config import create_node_info_dir
from node import Node, NodeService
from common.logger import log, add_logs_handler_file
from core import pre_shared_key
from web.local_server import LocalServerWrapper
from web.external_server import ExternalServerWrapper
from time import sleep


common.config.init_config(params.args.config_file)
common.file.initialise_file_managers()

create_node_info_dir()
add_logs_handler_file()
params.args.startup_args.append("--rpc-port")
params.args.startup_args.append(str(common.config.config_service.config.node_http_rpc_port))
params.args.startup_args.append("--psk-file")
params.args.startup_args.append(common.config.config_service.config.psk_file_path)
params.args.startup_args.append("--runner-port")
params.args.startup_args.append(str(common.config.config_service.config.local_server_port))
params.args.startup_args.append("--node-key-file")
params.args.startup_args.append(common.config.config_service.config.node_key_file_path)
node.node_service = NodeService(Node(params.args.startup_args))

try:
    log.info("Starting QMC runner...")
    if not common.file.psk_file_manager.exists():
        psk_obj = None
        while psk_obj is None:
            psk_obj = pre_shared_key.get_psk_from_peers()
            sleep(10)
        common.file.psk_file_manager.create(psk_obj.psk)
        common.file.psk_sig_file_manager.create(psk_obj.signature)

    node.node_service.current_node.start()

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
