from params import args
import time
from threading import Thread
from node import Node, NodeService
import node
from psk import fetch_from_peers, exists_psk_file, create_psk_file
from utils import log, add_logs_andler_file
from web import ExternalServerWrapper, LocalServerWrapper
import config
from config import create_directory_for_logs_and_other_files_of_node, Config, InvalidConfigurationFile, ConfigService


try:
    config.config_service = ConfigService(Config(args.config_file))
except InvalidConfigurationFile as e:
    log.error(e.message)
    exit()

create_directory_for_logs_and_other_files_of_node()
add_logs_andler_file()

args.startup_args.append("--psk-file")
args.startup_args.append(config.config_service.current_config.psk_file_path)
args.startup_args.append("--runner-port")
args.startup_args.append(str(config.config_service.current_config.local_server_port))
args.startup_args.append("--node-key-file")
args.startup_args.append(config.config_service.current_config.node_key_file_path)

node.node_service = NodeService(Node(args.startup_args))

try:
    log.info("Starting QMC runner...")
    if not exists_psk_file():
        # peer id ?
        psk = fetch_from_peers("12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc")
        create_psk_file(psk)

    # Wait until psk file is created
    while not exists_psk_file():
        time.sleep(1)

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
