import logging
import sys
import time
from threading import Thread

import node
import psk_file
from external_server import ExternalServerWrapper
from local_server import LocalServerWrapper
from node import Node, NodeService
from psk import fetch_from_peers
import psk_rotation_test
from config import config


logging.basicConfig(format='[%(asctime)s] %(levelname)s : %(message)s', level=logging.INFO)

startup_args = sys.argv[1:]

if startup_args[0] == 'test':
    psk_rotation_test.start_test()
else:
    startup_args.append("--psk-file")
    startup_args.append(config.config['psk_file_path'])
    startup_args.append("--runner-port")
    startup_args.append(str(config.config['local_server_port']))
    startup_args.append("--node-key-file")
    startup_args.append(config.config['node_key_file_path'])
    node.node_service = NodeService(Node(startup_args[2:]))

    try:
        logging.info("Starting QMC runner...")
        if not psk_file.exists():
            psk = fetch_from_peers()
            psk_file.create(psk)

        # Wait until psk file is created
        while not psk_file.exists():
            time.sleep(1)

        node.node_service.current_node.start()
        external_server = ExternalServerWrapper()
        external_thread = Thread(target=external_server.run, args=())
        logging.info("Starting external server...")
        external_thread.start()

        logging.info("Starting local server...")
        local_server = LocalServerWrapper()
        local_server.run()

    except Exception as e:
        logging.error(str(e))
    finally:
        logging.info("Closing QMC processes...")
        node.node_service.current_node.terminate()
