from config import config
import json
from flask import Flask, jsonify, Response
from psk import exists_psk_file, get_enc_key
from utils import xor, log


class ExternalServerWrapper:

    def __init__(self):
        self.external_server = Flask(__name__)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', get_psk, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.external_server.run(None, config.config["external_server_port"], False)


# TODO add peer authorizationS
def get_psk(peer_id):
    log.info(f"Fetching psk for peer with id: {peer_id}...")

    try:
        peer_config = config.config["peers"][peer_id]
    except KeyError:
        log.debug(f"{peer_id} not found - this peer is not configured")
        return Response(json.dumps({"message": "Peer not found"}), status=404, mimetype="application/json")

    if peer_config is None:
        log.debug(f"{peer_id} not found - this peer is not configured")
        return Response(json.dumps({"message": "Peer not found"}), status=404, mimetype="application/json")

    if not exists_psk_file():
        log.debug("Couldn't find psk file")
        return Response(json.dumps({"message": "Couldn't find psk file"}), status=404, mimetype="application/json")

    try:
        with open(config.abs_psk_file_path()) as file:
            psk_key = file.read()
        with open(config.abs_psk_sig_file_path()) as file:
            signature = file.read()

    except OSError as e:
        log.error(f"Invalid psk file: {e}")
        return Response(json.dumps({"message": "Invalid psk file"}), status=500, mimetype="application/json")

    try:
        key_id, qkd_key = get_enc_key(peer_config['qkd_addr'])
        xored_psk = xor(psk_key, qkd_key)
    except KeyError:
        return Response(json.dumps({"message": "Couldn't get enc key from QKD"}), status=500, mimetype="application/json")

    return jsonify({
        "key": xored_psk,
        "key_id": key_id,
        "signature": signature
    })
