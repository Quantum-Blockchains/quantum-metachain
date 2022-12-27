import json
import logging
from flask import Flask, jsonify, Response
import psk_file
from qkd import get_enc_key
from utils import xor


class ExternalServerWrapper():

    def __init__(self, config, **configs):
        self.external_server = Flask(__name__)
        self.config = config
        self.configs(**configs)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', self.get_psk, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def configs(self, **configs):
        for config, value in configs:
            self.external_server.config[config.upper()] = value

    def run(self):
        self.external_server.run(None, self.config.config["external_server_port"], False)

    # TODO add peer authorization
    def get_psk(self, peer_id):
        logging.info(f"Fetching psk for peer with id: {peer_id}...")

        peer_config = self.config.config["peers"][peer_id]
        if peer_config is None:
            logging.error(f"{peer_id} not found - this peer is not configured")
            return Response(json.dumps({"message": "Peer not found"}), status=404, mimetype="application/json")

        if not psk_file.exists(self.config):
            logging.error("Couldn't find psk file")
            return Response(json.dumps({"message": "Couldn't find psk file"}), status=404, mimetype="application/json")

        try:
            with open(self.config.abs_psk_file_path()) as file:
                psk_key = file.read()

        except OSError:
            logging.error("Couldn't open psk file")
            return Response(json.dumps({"message": "Couldn't open psk file"}), status=500, mimetype="application/json")

        key_id, qkd_key = get_enc_key(peer_config['qkd_addr'])
        xored_psk = xor(psk_key, qkd_key)

        return jsonify({
            "key": xored_psk,
            "key_id": key_id
        })
