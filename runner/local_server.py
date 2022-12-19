import logging

from flask import Flask, request, make_response

import node
import psk_file
from psk import fetch_from_qrng, fetch_from_peers
from auth import sign
from config import abs_node_key_file_path, abs_psk_sig_file_path

local_server = Flask(__name__)

logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)


@local_server.route("/psk", methods=['POST'])
def rotate_pre_shared_key():
    logging.info("Rotating pre-shared key...")
    body = request.get_json()
    is_local_peer = body["is_local_peer"]
    if is_local_peer:
        qrng_generated_psk = fetch_from_qrng()
        psk = sign(qrng_generated_psk, abs_node_key_file_path(), abs_psk_sig_file_path())

    else:
        psk = fetch_from_peers()

    psk_file.create(psk)

    node.node_service.current_node.restart()

    return make_response()
