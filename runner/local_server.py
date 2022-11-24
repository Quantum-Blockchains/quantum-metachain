from flask import Flask, request, make_response

import node
import psk_file

local_server = Flask(__name__)


@local_server.route("/psk", methods=['POST'])
def save_pre_shared_key():
    body = request.get_json()
    key = body["psk"]

    if is_valid_hex(key):
        psk_file.create(key)

    # TODO if psk not included in body, call other nodes, qkd, and verify psk
    node.node_service.current_node.restart()

    return make_response()


def is_valid_hex(psk):
    return psk and int(psk, 16) and len(psk) == 64
