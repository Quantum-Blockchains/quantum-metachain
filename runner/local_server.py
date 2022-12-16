import logging
from time import sleep

from flask import Flask, request, make_response

import node
import psk_file
from psk import fetch_from_qrng, fetch_from_peers
from config import config

local_server = Flask(__name__)

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)


@local_server.route("/psk", methods=['POST'])
def rotate_pre_shared_key():
    logging.info("Rotating pre-shared key...")
    body = request.get_json()
    is_local_peer = body["is_local_peer"]
    psk = fetch_from_qrng() if is_local_peer else fetch_from_peers()
    psk_file.create(psk)
    sleep(config["key_rotation_time"])

    node.node_service.current_node.restart()

    return make_response()
