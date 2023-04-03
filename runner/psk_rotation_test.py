import common.config
from common.logger import log
from common import crypto
import common.crypto
import requests
import time
import os
import subprocess
from os import path
import json
from web.qkd_mock_server import QkdMockServerWrapper
import multiprocessing
from multiprocessing import Process

multiprocessing.set_start_method("fork")

with open('test/tmp/alice/config_alice.json', "r") as f:
    config_alice = json.load(f, object_hook=common.config.custom_config_decoder)
with open('test/tmp/bob/config_bob.json', "r") as f:
    config_bob = json.load(f, object_hook=common.config.custom_config_decoder)
with open('test/tmp/charlie/config_charlie.json', "r") as f:
    config_charlie = json.load(f, object_hook=common.config.custom_config_decoder)
with open('test/tmp/dave/config_dave.json', "r") as f:
    config_dave = json.load(f, object_hook=common.config.custom_config_decoder)

nodes = [("alice", config_alice), ("bob", config_bob), ("charlie", config_charlie), ("dave", config_dave)]


def start_test():

    qkd = QkdMockServerWrapper()
    log.info("Starting qkd server...")
    qkd_server = Process(target=qkd.run)
    qkd_server.start()

    log.info("Starting test...")

    test = False

    process_alice = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/alice/config_alice.json",
         "ALICE"])

    process_bob = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/bob/config_bob.json", "BOB"])

    process_charlie = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/charlie/config_charlie.json",
         "CHARLIE"])

    process_dave = subprocess.Popen(
        ["python3", "runner_services_for_tests.py", "--config", "test/tmp/dave/config_dave.json",
         "DAVE"])

    time.sleep(10)

    try:
        block_number = 1
        for name, node_config in nodes:
            check_psk_rotation(name, node_config, block_number)
            block_number += 1

        test = True

    except Exception as e:
        test = False
        log.error("ERROR: " + str(e))
    finally:
        qkd_server.terminate()
        process_alice.terminate()
        process_bob.terminate()
        process_charlie.terminate()
        process_dave.terminate()

        for _, config in nodes:
            if path.exists(config.abs_log_node_file_path()):
                os.remove(config.abs_log_node_file_path())
            if path.exists(config.abs_psk_file_path()):
                os.remove(config.abs_psk_file_path())
            if path.exists(config.abs_psk_sig_file_path()):
                os.remove(config.abs_psk_sig_file_path())

        log.info("Closing QMC processes...")

        if test:
            log.info("Test: Successfully")
        else:
            log.info("Test: Not successfully")


def check_psk_rotation(signer_name, signer_config, block_number):
    send_psk_rotation_request(signer_config.local_server_port, signer_config.local_peer_id, True, block_number)
    sleep_until_file_exists(signer_config.abs_psk_file_path())

    for node_name, node_config in nodes:
        if node_config.local_peer_id in signer_config.peers:
            send_psk_rotation_request(node_config.local_server_port, signer_config.local_peer_id, False, block_number)
            sleep_until_file_exists(node_config.abs_psk_file_path())

    with open(signer_config.abs_psk_file_path(), 'r') as file:
        psk = file.read()

    with open(signer_config.abs_psk_sig_file_path(), 'r') as file:
        sig = file.read()

    for node_name, node_config in nodes:
        if node_name != signer_name:
            if not path.exists(node_config.abs_psk_file_path()):
                send_psk_rotation_request(node_config.local_server_port, signer_config.local_peer_id, False, block_number)
                sleep_until_file_exists(node_config.abs_psk_file_path())

            with open(node_config.abs_psk_file_path(), 'r') as file:
                psk_node = file.read()

            if psk_node != psk:
                log.error(f"{psk} =! {psk_node}")
                raise ValueError(f"{signer_name}'s and {node_name}'s keys are different")

            if not common.crypto.verify(f"{block_number}{psk_node}", bytes.fromhex(sig),
                                        common.crypto.to_public_from_peerid(signer_config.local_peer_id)):
                raise ValueError(f"({signer_name}) {node_name} psk verification failed.")
            else:
                log.info(f"({signer_name}) {node_name} psk verification successful")
    log.info(f"{signer_name}'s psk rotation successful")
    time.sleep(60)


def send_psk_rotation_request(runner_port, peer_id, is_local, block_number):
    url = f"http://localhost:{runner_port}/psk"
    data = {'peer_id': peer_id, 'is_local_peer': is_local, 'block_num': block_number}
    requests.post(url, json=data)


def sleep_until_file_exists(file_path):
    timestamp = time.time()
    while not path.exists(file_path):
        if time.time() - timestamp > 60:
            raise ValueError(f"No file at {file_path} after 1 minute")
        time.sleep(1)


start_test()
