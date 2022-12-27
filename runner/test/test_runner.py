import logging
from threading import Thread
from external_server import ExternalServerWrapper
from local_server import LocalServerWrapper
from config import Config
from node import NodeService, Node
import requests
import psk_file
import time
import os

def start_test():
    logging.info("Starting test...")

    test = True

    con_alice = Config('runner/test/config/config_alice.json')

    con_bob = Config('runner/test/config/config_bob.json')

    logging.info("Starting Alice test node...")
    node_service_alice = NodeService(Node(["python3", "runner/test_node.py", 'runner/test/config/config_alice.json']))
    node_service_alice.current_node.start()

    logging.info("Starting Bob test node...")
    node_service_bob = NodeService(Node(["python3", "runner/test_node.py", 'runner/test/config/config_bob.json']))
    node_service_bob.current_node.start()

    try:

        external_server_alice = ExternalServerWrapper(con_alice)
        external_thread_alice = Thread(target=external_server_alice.run, args=())
        logging.info("Starting Alice external server...")
        external_thread_alice.daemon = True
        external_thread_alice.start()

        local_server_alice = LocalServerWrapper(con_alice, node_service_alice)
        local_thread_alice = Thread(target=local_server_alice.run, args=())
        logging.info("Starting Alice local server...")
        local_thread_alice.daemon = True
        local_thread_alice.start()

        external_server_bob = ExternalServerWrapper(con_bob)
        external_thread_bob = Thread(target=external_server_bob.run, args=())
        logging.info("Starting Bob external server...")
        external_thread_bob.daemon = True
        external_thread_bob.start()

        local_server_bob = LocalServerWrapper(con_bob, node_service_bob)
        local_thread_bob = Thread(target=local_server_bob.run, args=())
        logging.info("Starting Bob local server...")
        local_thread_bob.daemon = True
        local_thread_bob.start()

        thread_1 = Thread(target=send_psk_rotation_request, args=(con_alice.config["local_server_port"], con_alice.config["local_peer_id"], True))
        thread_2 = Thread(target=send_psk_rotation_request, args=(con_bob.config["local_server_port"], con_alice.config["local_peer_id"], False))
        thread_1.start()
        thread_2.start()

        psk_alice = None
        psk_bob = None

        while not psk_file.exists(con_alice):
            time.sleep(1)

        with open(con_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        thread_1.join()

        while not psk_file.exists(con_bob):
            time.sleep(1)

        with open(con_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        thread_2.join()

        if psk_bob != psk_alice:
            test = False
            raise ValueError("Alice and Bob's keys are different")

        time.sleep(50)

        thread_3 = Thread(target=send_psk_rotation_request,
                          args=(con_alice.config["local_server_port"], con_bob.config["local_peer_id"], False))
        thread_4 = Thread(target=send_psk_rotation_request,
                          args=(con_bob.config["local_server_port"], con_bob.config["local_peer_id"], True))
        thread_3.start()
        thread_4.start()

        while not psk_file.exists(con_bob):
            time.sleep(1)

        with open(con_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        thread_4.join()

        while not psk_file.exists(con_alice):
            time.sleep(1)

        with open(con_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        thread_3.join()

        if psk_alice != psk_bob:
            test = False
            raise ValueError("Alice and Bob's keys are different")

    except Exception as e:
        logging.error("ERROR: " + str(e))
    finally:
        if psk_file.exists(con_alice):
            os.remove(con_alice.abs_psk_file_path())
        if psk_file.exists(con_bob):
            os.remove(con_bob.abs_psk_file_path())
        logging.info("Closing QMC processes...")
        node_service_alice.current_node.terminate()
        node_service_bob.current_node.terminate()
        if test:
            logging.info("Test: OK")
        else:
            logging.info("Test: ERROR")

def send_psk_rotation_request(runner_port, peer_id, is_local):
    url = "http://localhost:{port}/psk".format(port = runner_port)
    data = {'peer_id': peer_id, 'is_local_peer': is_local}
    requests.post(url, json=data)

