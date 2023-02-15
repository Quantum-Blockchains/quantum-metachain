import common.config
from common.config import Config
from common.logger import log
from common import crypto

import requests
import time
import os
import subprocess
from os import path
import json


with open('test/tmp/alice/config_alice.json', "r") as f:
    config_alice = json.load(f, object_hook=common.config.custom_config_decoder)
with open('test/tmp/bob/config_bob.json', "r") as f:
    config_bob = json.load(f, object_hook=common.config.custom_config_decoder)


def start_test():

    log.info("Starting test...")
    test = False

    process_alice = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/alice/config_alice.json", "ALICE"])
    process_bob = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/bob/config_bob.json", "BOB"])

    time.sleep(10)

    try:
        send_psk_rotation_request(config_alice.local_server_port, config_alice.local_peer_id, True)
        time.sleep(10)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        with open(config_alice.abs_psk_sig_file_path(), 'r') as file:
            sig_alice = file.read()

        with open(config_alice.abs_node_key_file_path(), 'r') as file:
            priv_key_alice = file.read()

            if not crypto.verify(psk_alice, bytes.fromhex(sig_alice), crypto.to_public(priv_key_alice)):
                test = False
                raise ValueError("Alice psk signing failed.")
            else:
                log.info("Alice signing successful")

        send_psk_rotation_request(config_bob.local_server_port, config_alice.local_peer_id, False)

        timestamp = time.time()

        while not path.exists(config_alice.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Alice did not generate a psk within a minute.")
            time.sleep(1)

        while not path.exists(config_bob.abs_psk_file_path()):
            if time.time() - timestamp > 60:
                test = False
                raise ValueError("Bob didn't get the psk within a minute.")
            time.sleep(1)

        with open(config_bob.abs_psk_file_path(), 'r') as file:
            psk_bob = file.read()

        if psk_bob != psk_alice:
            test = False
            log.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        if not crypto.verify(psk_bob, bytes.fromhex(sig_alice), crypto.to_public_from_peerid(config_alice.local_peer_id)):
            test = False
            raise ValueError("Bob psk verification failed.")
        else:
            log.info("Bob psk verification successful")

        time.sleep(70)

        send_psk_rotation_request(config_bob.local_server_port, config_bob.local_peer_id, True)
        time.sleep(5)
        send_psk_rotation_request(config_alice.local_server_port, config_bob.local_peer_id, False)

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
            log.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        test = True

    except Exception as e:
        log.error("ERROR: " + str(e))
    finally:
        process_alice.terminate()
        process_bob.terminate()
        if path.exists(config_alice.abs_log_node_file_path()):
            os.remove(config_alice.abs_log_node_file_path())
        if path.exists(config_bob.abs_log_node_file_path()):
            os.remove(config_bob.abs_log_node_file_path())
        if path.exists(config_alice.abs_psk_file_path()):
            os.remove(config_alice.abs_psk_file_path())
        if path.exists(config_bob.abs_psk_file_path()):
            os.remove(config_bob.abs_psk_file_path())
        if path.exists(config_alice.abs_psk_sig_file_path()):
            os.remove(config_alice.abs_psk_sig_file_path())
        if path.exists(config_bob.abs_psk_sig_file_path()):
            os.remove(config_bob.abs_psk_sig_file_path())
        log.info("Closing QMC processes...")
        if test:
            log.info("Test: Successfully")
        else:
            log.info("Test: Not successfully")


def send_psk_rotation_request(runner_port, peer_id, is_local):
    url = f"http://localhost:{runner_port}/psk"
    data = {'peer_id': peer_id, 'is_local_peer': is_local}
    requests.post(url, json=data)


start_test()
