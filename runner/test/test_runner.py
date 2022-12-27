import logging
from threading import Thread

from external_server import ExternalServerWrapper
from local_server import LocalServerWrapper
# from test.test_node import create_thread_node
from config import Config
from node import NodeService, Node

def start_test():
    logging.info("Starting test...")

    con_alice = Config('runner/test/config/config_alice.json')

    con_bob = Config('runner/test/config/config_bob.json')

    node_service_alice = NodeService(Node(["python3", "runner/test_node.py", 'runner/test/config/config_alice.json']))

    node_service_bob = NodeService(Node(["python3", "runner/test_node.py", 'runner/test/config/config_bob.json']))

    try:
        logging.info("Starting Alice test node...")
        node_service_alice.current_node.start()

        external_server_alice = ExternalServerWrapper(con_alice)
        external_thread_alice = Thread(target=external_server_alice.run, args=())
        logging.info("Starting Alice external server...")
        external_thread_alice.start()

        local_server_alice = LocalServerWrapper(con_alice, node_service_alice)
        local_thread_alice = Thread(target=local_server_alice.run, args=())
        logging.info("Starting Alice local server...")
        local_thread_alice.start()

        logging.info("Starting Alice test node...")
        node_service_bob.current_node.start()

        external_server_bob = ExternalServerWrapper(con_bob)
        external_thread_bob = Thread(target=external_server_bob.run, args=())
        logging.info("Starting Bob external server...")
        external_thread_bob.start()

        local_server_bob = LocalServerWrapper(con_bob, node_service_bob)
        # local_thread_bob = Thread(target=local_server_bob.run, args=())
        logging.info("Starting Bob local server...")
        # local_thread_bob.start()
        local_server_bob.run()

    except Exception as e:
        logging.error(str(e))
    finally:
        logging.info("Closing QMC processes...")
        node_service_alice.current_node.terminate()
        node_service_bob.current_node.terminate()


