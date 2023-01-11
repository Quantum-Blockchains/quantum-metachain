import logging
from config import Config
import requests
import time
import os
import subprocess
from os import path


def start_test():

    logging.info("Starting test...")

    test = False

    config_alice = Config('runner/config/config_alice.json')

    config_bob = Config('runner/config/config_bob.json')

    process_alice = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_alice.json", "ALICE"])

    process_bob = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_bob.json", "BOB"])

    time.sleep(10)

    try:

        send_psk_rotation_request(config_alice.config["local_server_port"], config_alice.config["local_peer_id"], True)
        time.sleep(5)
        send_psk_rotation_request(config_bob.config["local_server_port"], config_alice.config["local_peer_id"], False)

        timestamp = time.time()

        while not path.exists(config_alice.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Alice did not generate a psk within a minute.")
            time.sleep(1)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        while not path.exists(config_bob.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Bob didn't get the psk within a minute.")
            time.sleep(1)

        with open(config_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        if psk_bob != psk_alice:
            test = False
            logging.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        time.sleep(70)

        send_psk_rotation_request(config_bob.config["local_server_port"], config_bob.config["local_peer_id"], True)
        time.sleep(5)
        send_psk_rotation_request(config_alice.config["local_server_port"], config_bob.config["local_peer_id"], False)

        timestamp = time.time()

        while not path.exists(config_bob.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Bob did not generate a psk within a minute.")
            time.sleep(1)

        with open(config_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        while not path.exists(config_alice.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Alice didn't get the psk within a minute.")
            time.sleep(1)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        if psk_alice != psk_bob:
            test = False
            logging.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        test = True

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
    url = f"http://localhost:{runner_port}/psk"
    data = {'peer_id': peer_id, 'is_local_peer': is_local}
    requests.post(url, json=data)
