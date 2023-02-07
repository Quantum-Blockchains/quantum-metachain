import logging
from config import Config
from crypto import verify, to_public, to_public_from_peerid
import requests
import requests_mock
import socket
import time
import os
import subprocess
from os import path
from qkd_mock_server import QkdMockServerWrapper

config_alice = Config('runner/config/config_alice.json')
config_bob = Config('runner/config/config_bob.json')

def start_test():
    logging.info("Starting test...")

    test = False

    process_alice = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_alice.json", "ALICE"])

    process_bob = subprocess.Popen(
        ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_bob.json", "BOB"])
    
    # process_charlie = subprocess.Popen(
    #     ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_charlie.json", "CHARLIE"])
    
    # process_dave = subprocess.Popen(
    #     ["python3", "runner/runner_services_for_tests.py", "--config", "runner/config/config_dave.json", "DAVE"])

    # qkd_mock_server = QkdMockServerWrapper()
    # logging.info("Starting qkd mock server...")
    # qkd_mock_server.run()

    time.sleep(10)

    try:
        send_psk_rotation_request("alice", True)
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
                logging.info("Alice signing successful")

        send_psk_rotation_request("alice", False, "bob")
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
            logging.error(f"{psk_alice} =! {psk_bob}")
            raise ValueError("Alice and Bob's keys are different")

        if not verify(psk_bob, bytes.fromhex(sig_alice), to_public_from_peerid(config_alice.config["local_peer_id"])):
            test = False
            raise ValueError("Bob psk verification failed.")
        else:
            logging.info("Bob psk verification successful")

        time.sleep(70)

        send_psk_rotation_request("bob", True)
        time.sleep(5)
        send_psk_rotation_request("bob", False, "alice")

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
        # process_charlie.terminate()
        # process_dave.terminate()
        if path.exists(config_alice.abs_psk_file_path()):
            os.remove(config_alice.abs_psk_file_path())
        if path.exists(config_bob.abs_psk_file_path()):
            os.remove(config_bob.abs_psk_file_path())
        if path.exists(config_alice.abs_psk_sig_file_path()):
            os.remove(config_alice.abs_psk_sig_file_path())
        if path.exists(config_bob.abs_psk_sig_file_path()):
            os.remove(config_bob.abs_psk_sig_file_path())
        logging.info("Closing QMC processes...")
        if test:
            logging.info("Test: Successfully")
        else:
            logging.info("Test: Not successfully")


def send_psk_rotation_request(signer, is_local, verifier: str = None):
    config_signer = Config(f'runner/config/config_{signer}.json').config
    url = f"http://localhost:{config_signer['local_server_port']}/psk"
    data = {'peer_id': config_signer['local_peer_id'], 'is_local_peer': is_local}

    qkd_key = "0x1f1205c6a4ac0e3ff341ad6ea8f2945d5fedbd86e1301e6f146e7358feaf5b02"
    qkd_key_id = "ed1185e5-6223-415f-95fd-6364dcb2df32"
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"

    if not is_local:
        config_verifier = Config(f'runner/config/config_{verifier}.json').config
        with requests_mock.Mocker() as mock:
            create_qkd_response_mock(mock, verifier, config_signer['local_peer_id'], qkd_key, qkd_key_id)
            create_peer_response_mock(mock, signer, config_verifier['local_peer_id'], qkd_key, qkd_key_id, signature)

    requests.post(url, json=data)


# def mock_qkd_server_client():
#     try:
#         client_socket = socket.socket()
#         client_socket.connect((socket.gethostname(), 8182))
#         print(f"Qkd mock server connection successful.")
#     except Exception as e:
#         print(f"Couldn't connect to qkd mock server. Error: {e}")


def create_peer_response_mock(requests_mock, node_name, peer_id, key, key_id, signature):
    config = Config(f'runner/config/config_{node_name}.json').config
    server_addr = f"http://localhost:{config['external_server_port']}"
    print(f"FETCH FROM PEERS ADDR: {server_addr}")
    print(f"Verifier peer ID: {peer_id}")
    peers_response = {
        "key": key,
        "key_id": key_id,
        "signature": signature
    }

    get_psk_url = f"{server_addr}/peer/{peer_id}/psk"
    print(f"GET PSK MOCK URL: {get_psk_url}")
    requests_mock.get(get_psk_url, json=peers_response)


def create_qkd_response_mock(requests_mock, node_name, peer_id, key, key_id):
    config = Config(f'runner/config/config_{node_name}.json').config
    qkd_addr = config["peers"][peer_id]["qkd_addr"]
    qkd_reponse = {"keys": [{
        "key_ID": key_id,
        "key": key
    }]}
    requests_mock.get(f"{qkd_addr}/dec_keys?key_ID={key_id}", json=qkd_reponse)
