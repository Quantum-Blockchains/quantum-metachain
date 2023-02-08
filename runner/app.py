from config import config, create_directory_for_logs_and_other_files_of_node
import sys
import time
from threading import Thread
from node import Node, NodeService, write_logs_node_to_file
import node
from psk import get_psk_from_peers, exists_psk_file, create_psk_file
from utils import log, add_logs_andler_file
from web import ExternalServerWrapper, LocalServerWrapper
import signature_file


startup_args = sys.argv[1:]

create_directory_for_logs_and_other_files_of_node()
add_logs_andler_file()
startup_args.append("--psk-file")
startup_args.append(config.config['psk_file_path'])
startup_args.append("--runner-port")
startup_args.append(str(config.config['local_server_port']))
startup_args.append("--node-key-file")
startup_args.append(config.config['node_key_file_path'])
node.node_service = NodeService(Node(startup_args[2:]))

try:
    log.info("Starting QMC runner...")
    if not exists_psk_file():
        psk, signature = get_psk_from_peers()
        create_psk_file(psk)
        signature_file.create(signature)

    # Wait until psk file is created
    while not exists_psk_file():
        time.sleep(1)

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
