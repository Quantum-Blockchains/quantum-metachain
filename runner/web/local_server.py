from threading import Thread
from time import sleep

import node
from common.config import config_service
from flask import Flask, request, make_response, Response
from core import pre_shared_key
from common.logger import log
# from common.file import psk_file_manager, psk_sig_file_manager, node_key_file_manager
from common import crypto
import json
import common.config
import common.file
from node import write_logs_node_to_file


class LocalServerWrapper:

    def __init__(self):
        self.local_server = Flask(__name__)
        self.add_endpoint('/psk', 'rotate_pre_shared_key', start_thread_with_rotate_pre_shared_key, methods=['POST'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=None, *args, **kwargs):
        if methods is None:
            methods = ['GET']
        self.local_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.local_server.run(None, common.config.config_service.current_config.local_server_port, False)


def start_thread_with_rotate_pre_shared_key():
    body = request.get_json()
    thread = Thread(target=rotate_pre_shared_key, args=(body,))
    thread.start()
    return make_response()


def rotate_pre_shared_key(body):
    log.info("Rotating pre-shared key...")
    try:
        is_local_peer = body["is_local_peer"]
        peer_id = body["peer_id"]

    except KeyError:
        return Response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")

    if is_local_peer:
        psk = pre_shared_key.generate_psk_from_qrng()
        node_key = common.file.node_key_file_manager.read()
        signature = crypto.sign(psk, node_key).hex()
    else:
        psk, signature = pre_shared_key.get_psk_from_peers(peer_id)

    common.file.psk_file_manager.create(psk)
    common.file.psk_sig_file_manager.create(signature)
    sleep(common.config.config_service.current_config.key_rotation_time)

    node.node_service.current_node.restart()
    write_node_logs_thread = Thread(target=write_logs_node_to_file, args=())
    write_node_logs_thread.start()

    common.file.psk_sig_file_manager.remove()
