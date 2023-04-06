from threading import Thread
from time import sleep

import node
from flask import Flask, request, make_response, Response
from core import pre_shared_key
from common.logger import log
from common import crypto
import json
import common.config
import common.file
from web.error_handler import init_error_handlers


GET_PSK_WAITING_TIME = 1


class LocalServerWrapper:

    def __init__(self):
        self.local_server = Flask(__name__)
        init_error_handlers(self.local_server)
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
        block_number = body["block_num"]

    except KeyError:
        return Response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")

    if is_local_peer:
        psk = pre_shared_key.generate_psk_from_qrng()
        node_key = common.file.node_key_file_manager.read()
        data_for_sign = pre_shared_key.PskWithBlockNumber((block_number, psk))
        signature = crypto.sign(pre_shared_key.psk_with_block_number_to_string(data_for_sign), node_key).hex()
    else:
        get_psk_result = None

        while get_psk_result is None:
            get_psk_result = pre_shared_key.get_psk_from_peers(block_number, peer_id)
            sleep(GET_PSK_WAITING_TIME)
        psk, signature = get_psk_result

    common.file.psk_file_manager.create(psk)
    common.file.psk_sig_file_manager.create(signature)
    sleep(common.config.config_service.current_config.key_rotation_time)

    node.node_service.current_node.restart()

    common.file.psk_sig_file_manager.remove()
