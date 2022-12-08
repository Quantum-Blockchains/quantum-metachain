from flask import Flask, request, make_response

import node
import logging
import psk_file
from qrng import get_psk

local_server = Flask(__name__)

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)


@local_server.route("/psk", methods=['POST'])
def rotate_pre_shared_key():
    logging.info("Rotating pre-shared key...")
    body = request.get_json()
    is_local_peer = body["is_local_peer"]

    if is_local_peer:
        logging.info("Calling QRNG Api to get new PSK ...")
        key = get_psk()
        logging.info(key)
        psk_file.create(key)
    else:
        # TODO call other nodes, qkd, and verify psk
        pass

    node.node_service.current_node.restart()

    return make_response()
