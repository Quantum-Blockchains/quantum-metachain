import logging
from threading import Thread
from config import Config
import requests
import time
import os
import subprocess
from os import path


def start_test():

    logging.info("Starting test...")

    test = True

    config_alice = Config('runner/config/config_alice.json')

    config_bob = Config('runner/config/config_bob.json')

    process_alice = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_alice.json", "ALICE"])

    process_bob = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_bob.json", "BOB"])

    try:
        time.sleep(5)

        thread_send_request_to_local_server_alice = Thread(target=send_psk_rotation_request,
                                                           args=(config_alice.config["local_server_port"],
                                                                 config_alice.config["local_peer_id"], True))
        thread_send_request_to_local_server_bob = Thread(target=send_psk_rotation_request,
                                                         args=(config_bob.config["local_server_port"],
                                                               config_alice.config["local_peer_id"], False))

        thread_send_request_to_local_server_alice.start()

        time.sleep(5)

        thread_send_request_to_local_server_bob.start()

        psk_alice = None
        psk_bob = None
        timestamp = time.time()

        while not path.exists(config_alice.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                thread_send_request_to_local_server_alice.join()
                thread_send_request_to_local_server_bob.join()
                raise ValueError("Alice did not generate a psk within a minute.")
            time.sleep(1)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        while not path.exists(config_bob.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                thread_send_request_to_local_server_alice.join()
                thread_send_request_to_local_server_bob.join()
                test = False
                raise ValueError("Bob didn't get the psk within a minute.")
            time.sleep(1)

        with open(config_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        thread_send_request_to_local_server_alice.join()
        thread_send_request_to_local_server_bob.join()

        if psk_bob != psk_alice:
            test = False
            raise ValueError("Alice and Bob's keys are different")

        # time.sleep(50)

        thread_send_request_to_local_server_bob = Thread(target=send_psk_rotation_request,
                                                         args=(config_alice.config["local_server_port"],
                                                               config_bob.config["local_peer_id"], False))
        thread_send_request_to_local_server_alice = Thread(target=send_psk_rotation_request,
                                                           args=(config_bob.config["local_server_port"],
                                                                 config_bob.config["local_peer_id"], True))
        thread_send_request_to_local_server_bob.start()

        # time.sleep(5)

        thread_send_request_to_local_server_alice.start()

        timestamp = time.time()

        while not path.exists(config_bob.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                thread_send_request_to_local_server_alice.join()
                thread_send_request_to_local_server_bob.join()
                raise ValueError("Bob did not generate a psk within a minute.")
            time.sleep(1)

        with open(config_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        while not path.exists(config_alice.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                thread_send_request_to_local_server_alice.join()
                thread_send_request_to_local_server_bob.join()
                raise ValueError("Alice didn't get the psk within a minute.")
            time.sleep(1)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        thread_send_request_to_local_server_bob.join()
        thread_send_request_to_local_server_alice.join()

        if psk_alice != psk_bob:
            test = False
            raise ValueError("Alice and Bob's keys are different")

    except Exception as e:
        logging.error("ERROR: " + str(e))
    finally:
        process_alice.terminate()
        process_bob.terminate()
        if path.exists(config_alice.abs_psk_file_path()):
            os.remove(config_alice.abs_psk_file_path())
        if path.exists(config_bob.abs_psk_file_path()):
            os.remove(config_bob.abs_psk_file_path())
        logging.info("Closing QMC processes...")
        if test:
            logging.info("Test: Successfully")
        else:
            logging.info("Test: Not successfully")


def send_psk_rotation_request(runner_port, peer_id, is_local):
    url = "http://localhost:{port}/psk".format(port=runner_port)
    data = {'peer_id': peer_id, 'is_local_peer': is_local}
    requests.post(url, json=data)
