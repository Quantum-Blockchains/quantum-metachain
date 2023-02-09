from threading import Thread

import node
import common.config
import common.file
import params
from common.config import create_node_info_dir, InvalidConfigurationFile
from node import Node, NodeService, write_logs_node_to_file
from common.logger import log, add_logs_handler_file
from core import pre_shared_key
from web import ExternalServerWrapper, LocalServerWrapper


try:
    common.config.config_service = common.config.ConfigService(common.config.Config(params.args.config_file))
except InvalidConfigurationFile as e:
    log.error(e.message)
    exit()

common.file.psk_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_file_path())
common.file.node_key_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_node_key_file_path())
common.file.psk_sig_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_sig_file_path())

create_node_info_dir()
add_logs_handler_file()
params.args.startup_args.append("--psk-file")
params.args.startup_args.append(common.config.config_service.current_config.psk_file_path)
params.args.startup_args.append("--runner-port")
params.args.startup_args.append(str(common.config.config_service.current_config.local_server_port))
params.args.startup_args.append("--node-key-file")
params.args.startup_args.append(common.config.config_service.current_config.node_key_file_path)
node.node_service = NodeService(Node(params.args.startup_args))

try:
    log.info("Starting QMC runner...")
    if not common.file.psk_file_manager.exists():
        psk, signature = pre_shared_key.get_psk_from_peers()
        common.file.psk_file_manager.create(psk)
        common.file.psk_sig_file_manager.create(signature)

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
