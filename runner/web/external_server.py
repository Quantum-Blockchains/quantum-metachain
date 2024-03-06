from flask import Flask, jsonify

import common.config
import common.file
from common import exceptions
from common.logger import log
from core import onetimepad
from core.qkd.provider_factory import get_qkd_provider
from web.error_handler import init_error_handlers


class ExternalServerWrapper:

    def __init__(self):
        self.external_server = Flask(__name__)
        init_error_handlers(self.external_server)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', get_psk, methods=['GET'])
        self.add_endpoint('/search_node/<peer_id>', 'search_node', search_node, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=None, *args, **kwargs):
        if methods is None:
            methods = ['GET']
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
         self.external_server.run("0.0.0.0", common.config.config_service.config.external_server_port, False,
                                         threaded=True)


# TODO add peer authorizationS
def get_psk(peer_id):
    log.info(f"Fetching psk for peer with id: {peer_id}...")
    peer_config = common.config.config_service.config.peers.get(peer_id)
    if peer_config is None or peer_config["qkd"] is None:
        log.warning(f"Peer with id = {peer_id} is not configured")
        raise exceptions.PeerMisconfiguredError

    if not common.file.psk_file_manager.exists() or not common.file.psk_sig_file_manager.exists():
        log.warning("Couldn't find psk or signature file")
        raise exceptions.PSKNotFoundError

    psk = common.file.psk_file_manager.read()
    psk_sig = common.file.psk_sig_file_manager.read()
    qkd_provider = get_qkd_provider(peer_config['qkd'])
    key_id, qkd_key = qkd_provider.get_enc_key()
    xored_psk = onetimepad.encrypt(psk, qkd_key)

    return jsonify({
        "key": xored_psk,
        "key_id": key_id,
        "signature": psk_sig
    })

def search_node(peer_id):
    log.info("Add new peer to config...")
    if peer_id in common.config.config_service.config.peers.keys():
        peer_config = common.config.config_service.config.peers.get(peer_id)
        return jsonify({
            "found": True,
            "external_server_address": peer_config["server_addr"],
            "peers": []
        })
    else:
        addr_list = []
        for peer_id in common.config.config_service.config.peers.keys():
            peer_config = common.config.config_service.config.peers.get(peer_id)
            addr_list.append(peer_config["server_addr"])
        return jsonify({
            "found": False,
            "external_server_address": "",
            "peers": addr_list
        })
