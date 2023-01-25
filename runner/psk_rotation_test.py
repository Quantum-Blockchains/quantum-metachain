from config import Config
from utils import log, verify, to_public, to_public_from_peerid
import requests
import time
import os
import subprocess
from os import path


def start_test():

    log.info("Starting test...")

    test = False

    config_alice = Config('runner/test/tmp/alice/config_alice.json')

    config_bob = Config('runner/test/tmp/bob/config_bob.json')

    process_alice = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/test/tmp/alice/config_alice.json", "ALICE"])

    process_bob = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/test/tmp/bob/config_bob.json", "BOB"])

    time.sleep(10)

    try:

        send_psk_rotation_request(config_alice.config["local_server_port"], config_alice.config["local_peer_id"], True)
        time.sleep(10)

        with open(config_alice.abs_psk_file_path(), 'r') as file:
            psk_alice = file.read()

        with open(config_alice.abs_psk_sig_file_path(), 'r') as file:
            sig_alice = file.read()

        with open(config_alice.abs_node_key_file_path(), 'r') as file:
            priv_key_alice = file.read()

            if not verify(psk_alice, bytes.fromhex(sig_alice), to_public(priv_key_alice)):
                test = False
                raise ValueError("Alice psk signing failed.")
            else:
                log.info("Alice signing successful")

        send_psk_rotation_request(config_bob.config["local_server_port"], config_alice.config["local_peer_id"], False)

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

        if not verify(psk_bob, bytes.fromhex(sig_alice), to_public_from_peerid(config_alice.config["local_peer_id"])):
            test = False
            raise ValueError("Bob psk verification failed.")
        else:
            log.info("Bob psk verification successful")

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
            log.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        test = True

    except Exception as e:
        log.error("ERROR: " + str(e))
    finally:
        process_alice.terminate()
        process_bob.terminate()
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
