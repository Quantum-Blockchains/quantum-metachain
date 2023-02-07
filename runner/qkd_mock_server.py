import logging
from os import path
from flask import Flask, jsonify
from config import Config

config_alice = Config('runner/config/config_alice.json')
config_bob = Config('runner/config/config_bob.json')
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")


class QkdMockServerWrapper:

    def __init__(self):


        self.qkd_mock_server = Flask(__name__)
        logging.getLogger("werkzeug").setLevel("WARNING")
        self.add_endpoint('/mock_qkd/<node_name>/<peer_id>', 'get_mock_qkd_resp', get_mock_qkd_resp, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.qkd_mock_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.qkd_mock_server.run(None, 8182, True)


def get_mock_qkd_resp():
    # sleep(1)
    logging.info(f"Mock qkd get successful")
    return jsonify({
        "key": "0x1f1205c6a4ac0e3ff341ad6ea8f2945d5fedbd86e1301e6f146e7358feaf5b02",
        "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "signature": "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    })


# def create_peer_response_mock(node_name, peer_id):
#     config = Config(f'runner/config/config_{node_name}.json')
#     server_addr = f"http://localhost:{config['external_server_port']}"
#     peers_response = {
#         "key": "0x1f1205c6a4ac0e3ff341ad6ea8f2945d5fedbd86e1301e6f146e7358feaf5b02",
#         "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
#         "signature": "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
#     }

#     get_psk_url = f"{server_addr}/peer/{peer_id}/psk"
#     requests_mock.get(get_psk_url, json=peers_response)


# def create_qkd_response_mock(node_name, peer_id):
#     config = Config(f'runner/config/config_{node_name}.json')
#     qkd_addr = config.peers[peer_id]["qkd_addr"]
#     qkd_reponse = {"keys": [{
#         "key_ID": "ed1185e5-6223-415f-95fd-6364dcb2df32",
#         "key": "LH8sve7mz7ifkzjgmu/jdVtdjkDbMynHmrId09b2Xd0="
#     }]}
#     requests_mock.get(f"{qkd_addr}/dec_keys?key_ID=ed1185e5-6223-415f-95fd-6364dcb2df32", json=qkd_reponse)
