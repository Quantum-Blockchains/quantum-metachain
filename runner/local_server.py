import logging
from time import sleep
from flask import Flask, request, make_response
import psk_file
from psk import fetch_from_qrng, fetch_from_peers
from config import config
import node
from threading import Thread


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
    psk = fetch_from_qrng() if is_local_peer else fetch_from_peers()
    psk_file.create(psk)
    sleep(config.config["key_rotation_time"])

    node.node_service.current_node.restart()
