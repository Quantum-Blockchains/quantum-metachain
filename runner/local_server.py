import logging

import requests
from flask import Flask, request, make_response

import node
import psk_file
from qrng import get_psk
from config import config
from qkd import get_dec_key
from utils import xor

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
        logging.debug(f"Generated key: {key}")
        psk_file.create(key)
    else:
        peers = config["peers"]
        for peer_id, peer in peers.items():
            get_psk_url = f"{peer['server_addr']}/peer/{peer_id}/psk"
            get_psk_response = requests.get(get_psk_url).json()
            _, qkd_key = get_dec_key(peer["qkd_addr"], get_psk_response["key_id"])
            psk = xor(get_psk_response["key"], qkd_key)

            # TODO fetch psk from all peers in config, compare, verify and choose a valid one (TBD)
            psk_file.create(psk)

    node.node_service.current_node.restart()

    return make_response()
