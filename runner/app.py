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
from test import test_runner
from config import Config

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)

startup_args = sys.argv[1:]

if startup_args[0] == 'test':
    test_runner.start_test()
else:
    path_config = startup_args[0]
    config = Config(path_config)
    startup_args.append("--psk-file")
    startup_args.append(config.config['psk_file_path'])
    node.node_service = NodeService(Node(startup_args[1:]))

    try:
        logging.info("Starting QMC runner...")
        if not psk_file.exists(config):
            psk = fetch_from_peers(config.config)
            psk_file.create(psk, config)

        # Wait until psk file is created
        while not psk_file.exists(config):
            time.sleep(1)

        node.node_service.current_node.start()

        external_server = ExternalServerWrapper(config)
        external_thread = Thread(target=external_server.run, args=())
        logging.info("Starting external server...")
        external_thread.start()

        logging.info("Starting local server...")
        local_server = LocalServerWrapper(config, node.node_service)
        local_server.run()

    except Exception as e:
        logging.error(str(e))
    finally:
        logging.info("Closing QMC processes...")
        node.node_service.current_node.terminate()
