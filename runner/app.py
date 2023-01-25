from config import config
import sys
import time
from threading import Thread

from node import Node, NodeService
import node
from psk import fetch_from_peers, exists_psk_file, create_psk_file
import psk_rotation_test
from utils import log, addLogsHandlerFile
from web import ExternalServerWrapper, LocalServerWrapper


startup_args = sys.argv[1:]

if startup_args[0] == 'test':
    psk_rotation_test.start_test()
else:
    addLogsHandlerFile()
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
