import logging
from threading import Thread
from time import sleep

from flask import Flask, request, make_response

import node
import psk_file
import signature_file
from config import config
from crypto import sign
from psk import generate_psk_from_qrng, get_psk_from_peers


class LocalServerWrapper:

    def __init__(self):
        self.local_server = Flask(__name__)
        logging.getLogger("werkzeug").setLevel("WARNING")
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
    logging.info("Rotating pre-shared key...")
    is_local_peer = body["is_local_peer"]
    peer_id = body["peer_id"]
    if is_local_peer:
        psk = generate_psk_from_qrng()
        with open(config.abs_node_key_file_path()) as file:
            node_key = file.read()
            signature = sign(psk, node_key).hex()
    else:
        psk, signature = get_psk_from_peers(peer_id)

    psk_file.create(psk)
    signature_file.create(signature)

    sleep(config.config["key_rotation_time"])

    node.node_service.current_node.restart()
