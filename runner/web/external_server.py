from flask import Flask, jsonify
from core import qkd, onetimepad
from common.logger import log
import common.config
import common.file
from web.error_handler import init_error_handlers
from common import exceptions


class ExternalServerWrapper:

    def __init__(self):
        self.external_server = Flask(__name__)
        init_error_handlers(self.external_server)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', get_psk, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=None, *args, **kwargs):
        if methods is None:
            methods = ['GET']
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.external_server.run("0.0.0.0", common.config.config_service.config.external_server_port, False)


# TODO add peer authorizationS
def get_psk(peer_id):
    log.info(f"Fetching psk for peer with id: {peer_id}...")
    peer_config = common.config.config_service.config.peers.get(peer_id)
    if peer_config is None or peer_config["qkd_addr"] is None:
        log.warning(f"Peer with id = {peer_id} is not configured")
        raise exceptions.PeerMisconfiguredError

    if not common.file.psk_file_manager.exists() or not common.file.psk_sig_file_manager.exists():
        log.warning("Couldn't find psk or signature file")
        raise exceptions.PSKNotFoundError

    psk = common.file.psk_file_manager.read()
    psk_sig = common.file.psk_sig_file_manager.read()
    qkd_cert_path = peer_config["qkd_cert_path"]
    qkd_cert_key = peer_config["qkd_cert_key_path"]
    key_id, qkd_key = qkd.get_enc_key(peer_config['qkd_addr'], qkd_cert_path, qkd_cert_key)
    xored_psk = onetimepad.encrypt(psk, qkd_key)

    return jsonify({
        "key": xored_psk,
        "key_id": key_id,
        "signature": psk_sig
    })
