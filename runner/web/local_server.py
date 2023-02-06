import os
from threading import Thread
from time import sleep

import node
from config import config
from flask import Flask, request, make_response
from psk import generate_psk_from_qrng, get_psk_from_peers, create_psk_file, create_signature_file
from utils import sign, log


class LocalServerWrapper:

    def __init__(self):
        self.local_server = Flask(__name__)
        self.add_endpoint('/psk', 'rotate_pre_shared_key', start_thread_with_rotate_pre_shared_key, methods=['POST'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.local_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.local_server.run(None, config.config["local_server_port"], False)


def start_thread_with_rotate_pre_shared_key():
    body = request.get_json()
    thread = Thread(target=rotate_pre_shared_key, args=(body,))
    thread.start()
    return make_response()


def rotate_pre_shared_key(body):
    log.info("Rotating pre-shared key...")
    is_local_peer = body["is_local_peer"]
    peer_id = body["peer_id"]
    if is_local_peer:
        psk = generate_psk_from_qrng()

        with open(config.abs_node_key_file_path()) as file:
            node_key = file.read()
            signature = sign(psk, node_key)

        with open(config.abs_psk_sig_file_path(), 'w') as file:
            file.write(signature.hex())

    else:
        psk, signature = get_from_peers(peer_id)

    create_psk_file(psk)
    create_signature_file(signature)
    sleep(config.config["key_rotation_time"])

    node.node_service.current_node.restart()
    write_node_logs_thread = Thread(target=node.write_logs_node_to_file, args=())
    write_node_logs_thread.start()

    if os.path.exists(config.abs_psk_sig_file_path()):
        os.remove(config.abs_psk_sig_file_path())
