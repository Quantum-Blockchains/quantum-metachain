import logging
from time import sleep
from flask import Flask, request, make_response
import psk_file
from psk import fetch_from_qrng, fetch_from_peers
from config import config
import node


class LocalServerWrapper():

    def __init__(self, **configs):
        self.local_server = Flask(__name__)
        logging.getLogger("werkzeug").setLevel("WARNING")
        self.configs(**configs)
        self.add_endpoint('/psk', 'rotate_pre_shared_key', self.rotate_pre_shared_key, methods=['POST'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.local_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def configs(self, **configs):
        for con, value in configs:
            self.local_server.config[con.upper()] = value

    def run(self):
        self.local_server.run(None, config.config["local_server_port"], False)

    def rotate_pre_shared_key(self):
        logging.info("Rotating pre-shared key...")
        body = request.get_json()
        is_local_peer = body["is_local_peer"]
        psk = fetch_from_qrng() if is_local_peer else fetch_from_peers()
        psk_file.create(psk)
        sleep(config.config["key_rotation_time"])

        node.node_service.current_node.restart()

        return make_response()
