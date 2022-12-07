import logging

from flask import Flask, jsonify, Response

import psk_file
from config import config
from qkd import get_enc_key
from utils import xor

external_server = Flask(__name__)


# TODO add peer authorization
@external_server.route("/peer/<peer_id>/psk", methods=['GET'])
def get_psk(peer_id):
    logging.info(f"Fetching psk for peer with id: {peer_id}...")

    peer_config = config["peers"][peer_id]
    if peer_config is None:
        logging.error(f"{peer_id} not found - this peer is not configured")
        return Response("{'error': 'Peer not found'}", status=404, mimetype="application/json")

    if not psk_file.exists():
        logging.error("Couldn't find psk file")
        return Response("{'error': 'Couldn't find psk file'}", status=404, mimetype="application/json")

    try:
        with open(config["psk_file_path"]) as file:
            psk_key = file.read()

    except OSError:
        logging.error("Couldn't open psk file")
        return Response("{'error': 'Couldn't open psk file'}", status=500, mimetype="application/json")

    key_id, qkd_key = get_enc_key(peer_config['qkd_addr'])
    xored_psk = xor(psk_key, qkd_key)

    return jsonify({
        "key": xored_psk,
        "key_id": key_id
    })
