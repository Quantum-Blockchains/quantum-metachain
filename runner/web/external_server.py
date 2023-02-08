from common.config import config
import json
from flask import Flask, jsonify, Response
from core import qkd
from common.logger import log
from common.file import psk_file_manager, psk_sig_file_manager
from common import crypto


class ExternalServerWrapper:

    def __init__(self):
        self.external_server = Flask(__name__)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', get_psk, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=None, *args, **kwargs):
        if methods is None:
            methods = ['GET']
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.external_server.run(None, config.config["external_server_port"], False)


# TODO add peer authorizationS
def get_psk(peer_id):
    log.info(f"Fetching psk for peer with id: {peer_id}...")
    peer_config = config.config["peers"].get(peer_id)
    if peer_config is None or peer_config["qkd_addr"] is None:
        log.warning(f"Peer with id = {peer_id} is not configured")
        return Response(json.dumps({"message": "Peer is not configured"}), status=404, mimetype="application/json")

    if not psk_file_manager.exists() or not psk_sig_file_manager.exists():
        log.warning("Couldn't find psk or signature file")
        return Response(json.dumps({"message": "Pre shared key not found"}), status=404, mimetype="application/json")

    psk = psk_file_manager.read()
    psk_sig = psk_sig_file_manager.read()
    key_id, qkd_key = qkd.get_enc_key(peer_config['qkd_addr'])
    xored_psk = crypto.xor(psk, qkd_key)

    return jsonify({
        "key": xored_psk,
        "key_id": key_id,
        "signature": psk_sig
    })