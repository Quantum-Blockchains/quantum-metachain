from flask import Flask, jsonify, request, make_response, send_file, send_from_directory

import common.config
import common.file
from common import exceptions
from common.logger import log
from core import onetimepad
from core.qkd.provider_factory import get_qkd_provider
from web.error_handler import init_error_handlers
from substrateinterface import SubstrateInterface
from scalecodec import ScaleBytes
import json
from os import path, mkdir, makedirs
from urllib.parse import urlparse
from substrateinterface import Keypair, KeypairType
from substrateinterface.exceptions import SubstrateRequestException


class ExternalServerWrapper:

    def __init__(self):
        self.external_server = Flask(__name__)
        init_error_handlers(self.external_server)
        self.add_endpoint('/peer/<peer_id>/psk', 'get_psk', get_psk, methods=['GET'])
        self.add_endpoint('/search_node/<peer_id>', 'search_node', search_node, methods=['GET'])
        self.add_endpoint('/get_peers_for_node/<peer_id>', 'get_peers_for_node',
                          get_peers_for_node, methods=['GET'])
        self.add_endpoint('/data_qkd_exchange', 'data_qkd_exchange', data_qkd_exchange,
                          methods=['POST'])
        self.add_endpoint('/get_number_block_for_restart', 'get_number_block_for_restart',
                          get_number_block_for_restart, methods=['GET'])
        self.add_endpoint('/get_current_number_block', 'get_current_number_block', get_current_number_block,
                          methods=['GET'])
        self.add_endpoint('/targets_exchange', 'targets_exchange', targets_exchange,
                          methods=['POST'])

        self.add_endpoint('/schema/<id>', 'get_shema', get_schema, methods=['GET'])
        self.add_endpoint('/schema', 'set_schema', set_schema, methods=['POST'])
        self.add_endpoint('/credential-definition', 'set_credential_definition', set_credential_definition, methods=['POST'])
        self.add_endpoint('/credential-definition/<id>', 'get_credential_definition', get_credential_definition, methods=['GET'])
        self.add_endpoint('/revocation-registry-definition', 'set_revocation_registry_definition', set_revocation_registry_definition, methods=['POST'])
        self.add_endpoint('/revocation-registry-definition/<id>', 'get_revocation_registry_definition', get_revocation_registry_definition, methods=['GET'])
        self.add_endpoint('/revocation-list', 'set_revocation_list', set_revocation_list, methods=['POST'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=None, *args, **kwargs):
        if methods is None:
            methods = ['GET']
        self.external_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        if common.config.config_service.config.external_url_scheme == 'https':
            cert = path.join(common.config.node_dir, common.config.EXTERNAL_CERT_PATH)
            key = path.join(common.config.node_dir, common.config.EXTERNAL_KEY_PATH)
            self.external_server.run("0.0.0.0", common.config.config_service.config.external_server_port, False,
                                         threaded=True, ssl_context=(cert, key))
        else:
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
    log.info("Search peer...")
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


def get_peers_for_node(peer_id):
    log.info("Get peers for node...")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")

    hypercube_nodes = ws_provider.query(
        module="Hypercube",
        storage_function="Peers",
        params=[],
    )

    if peer_id not in hypercube_nodes:
        log.warning("Peer is not in hypercube.")
        raise exceptions.PeerIsNotInHypercube

    encode_peer = ws_provider.encode_scale("Vec<u8>", peer_id)

    peers_bytes = ws_provider.rpc_request(
        method="state_call",
        params=["HypercubeApi_links", encode_peer.to_hex()])["result"]

    peers = ws_provider.decode_scale("Vec<Vec<u8>>", ScaleBytes(peers_bytes))

    return jsonify({
        "peers": peers
    })


def targets_exchange():
    log.info("Target exchange...")
    if 'target' not in request.files:
        raise exceptions.NoTargetReceived
    target = request.files['target']
    if target.filename == ():
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")
    if target:
        if not path.exists(path.join(common.config.node_dir, 'pqkd/targets')):
            makedirs(path.join(common.config.node_dir, 'pqkd/targets'))
        target.save(path.join(common.config.node_dir, f'pqkd/targets/{target.filename}'))
    return send_file(path.join(common.config.node_dir, f'pqkd/{common.config.config_service.config.local_qkd_target}'),
                     download_name=path.basename(path.join(common.config.node_dir, f'pqkd/{common.config.config_service.config.local_qkd_target}')))


def data_qkd_exchange():
    # TODO sdzielac prowierki!!

    body = request.get_json()
    # body = json.loads(request.form['json'])
    try:
        peer_id = body["peer_id"]
        qkd_name = body["qkd_name"]
        server_addr = body["server_addr"]
    except KeyError:
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")

    log.info(f"Data qkd exchange with peer {peer_id}")
    url_pqkd = urlparse(common.config.config_service.config.local_qkd_url)
    if url_pqkd.scheme == 'https':
        cert = common.config.PQKD_CERT_PATH
        key = common.config.PQKD_KEY_PATH
    else:
        cert = ''
        key = ''
    qkd_info = {
        "qkd": {
            "provider": "etsi014",
            "url": common.config.config_service.config.local_qkd_url + "/api/v1/keys/" + qkd_name,
            "client_cert_path": cert,
            "cert_key_path":  key
        },
        "server_addr": server_addr
    }
    common.config.config_service.config.peers[peer_id] = qkd_info
    try:
        common.file.config_file_manager.remove()
    except FileNotFoundError as err:
        log.error(f"Error: {err}")
    common.file.config_file_manager.create(common.config.config_service.config.to_json())
    return jsonify({
        "peer_id": common.config.config_service.config.local_peer_id,
        "qkd_name": common.config.config_service.config.local_qkd_name,
        "server_addr": f'{common.config.config_service.config.external_url_scheme}://{common.config.config_service.config.public_ip}:{str(common.config.config_service.config.external_server_port)}'
    })


def get_number_block_for_restart():
    log.info("Get number block for restart...")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")

    block_for_restart = ws_provider.query(
        module="OcwPsk",
        storage_function="NumBlockForRestart",
        params=[]
    )

    return jsonify(({
        "num_block_for_restart": block_for_restart.value
    }))


def get_current_number_block():
    log.info("Get current number block...")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")

    block = ws_provider.get_block()

    return jsonify(({
        "current_block": block["header"]["number"]
    }))


def get_schema(id):
    log.info(f'Get shema. Schema id: {id}')
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    schema = ws_provider.query(
        module="Did",
        storage_function="Schemas",
        params=[id]
    )
    if schema == None:
        schema_json = {}
    else:
        schema_json = {
            "schema_id": schema.value["schema_id"],
            "issuer_id": schema.value["issuer_id"],
            "attr_names": schema.value["attr_names"],
            "name": schema.value["name"],
            "version": schema.value["version"],
            "ver": schema.value["ver"]
        }

    return jsonify(({
        "schema": schema_json
    }))


def set_schema():
    log.info('Set shema.')

    body = request.get_json()
    try:
        schema = body["schema"]
    except KeyError:
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")
   
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    mnemonic = "bone laugh knife column endorse despair rail track lend hope tribe quote"
    keypair = Keypair.create_from_mnemonic(mnemonic, crypto_type=KeypairType.ED25519)
    
    
    call = ws_provider.compose_call(
        call_module="Did",
        call_function="create_schema",
        call_params={
            "schema":  schema
        },
    )
    extrinsic = ws_provider.create_signed_extrinsic(call=call, keypair=keypair)
    try:
        receipt = ws_provider.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        print(
            "Extrinsic '{}' sent and included in block '{}'".format(
                receipt.extrinsic_hash, receipt.block_hash
            )
        )
        return jsonify(({
            "extrinsic_hash": receipt.extrinsic_hash,
            "block_hash": receipt.block_hash,
            "error": False,
            "message_error": ""
        })) 
    except SubstrateRequestException as e:
        print("Failed to send: {}".format(e))
        return jsonify(({
            "extrinsic_hash": "",
            "block_hash": "",
            "error": True,
            "message_error": "Failed to send: {}".format(e)
        }))


def set_credential_definition():
    log.info('Set credential definition.')
    body = request.get_json()
    try:
        cred_def = body["cred_def"]
    except KeyError:
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    mnemonic = "bone laugh knife column endorse despair rail track lend hope tribe quote"
    keypair = Keypair.create_from_mnemonic(mnemonic, crypto_type=KeypairType.ED25519)
    
    tmp = list()
    for i in cred_def["value"]["primary"]["r"]:
        tmp.append({"name": i, "value": cred_def["value"]["primary"]["r"][i]})
    cred_def["value"]["primary"]["r"] = tmp
    if not "revocation" in cred_def["value"]:
        cred_def["value"]["revocation"] = None
    
    call = ws_provider.compose_call(
        call_module="Did",
        call_function="create_credential_definition",
        call_params={
            "cred_def": cred_def
        },
    )
    extrinsic = ws_provider.create_signed_extrinsic(call=call, keypair=keypair)
    try:
        receipt = ws_provider.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        print(
            "Extrinsic '{}' sent and included in block '{}'".format(
                receipt.extrinsic_hash, receipt.block_hash
            )
        )
        return jsonify(({
            "extrinsic_hash": receipt.extrinsic_hash,
            "block_hash": receipt.block_hash,
            "error": False,
            "message_error": ""
        })) 
    except SubstrateRequestException as e:
        print("Failed to send: {}".format(e))
        return jsonify(({
            "extrinsic_hash": "",
            "block_hash": "",
            "error": True,
            "message_error": "Failed to send: {}".format(e)
        }))


def get_credential_definition(id):
    log.info(f'Get credential definition. Id: {id}')
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    cred_def = ws_provider.query(
        module="Did",
        storage_function="CredentialDefinitions",
        params=[id]
    )
    if cred_def == None:
        cred_def_json = {}
    else:
        cred_def_json = {
            "id": cred_def.value["cred_def_id"],
            "schemaId": cred_def.value["schema_id"],
            "type": cred_def.value["ttype"],
            "tag": cred_def.value["tag"],
            "value": cred_def.value["value"],
            "ver": cred_def.value["ver"]
        }
        tmp = {}
        for i in cred_def_json["value"]["primary"]["r"]:
            print(5)
            print(i)
            tmp[i["name"]] = i["value"]
        cred_def_json["value"]["primary"]["r"] = tmp
        if cred_def_json["value"]["revocation"] is None:
            del cred_def_json["value"]["revocation"]
    return jsonify(({
        "credential-definition": cred_def_json
    }))


def set_revocation_registry_definition():
    log.info('Set revocation registry definition.')
    body = request.get_json()
    try:
        rev_reg_def = body["rev_reg_def"]
    except KeyError:
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    mnemonic = "bone laugh knife column endorse despair rail track lend hope tribe quote"
    keypair = Keypair.create_from_mnemonic(mnemonic, crypto_type=KeypairType.ED25519)

    tmp = str(rev_reg_def["value"]["public_keys"])
    # for i in rev_reg_def["value"]["public_keys"]:
    #     tmp.append({"name": i, "value": rev_reg_def["value"]["public_keys"][i]})
    rev_reg_def["value"]["public_keys"] = tmp

    call = ws_provider.compose_call(
        call_module="Did",
        call_function="create_revocation_registry_definition",
        call_params={
            "rev_reg_def": rev_reg_def
        },
    )
    extrinsic = ws_provider.create_signed_extrinsic(call=call, keypair=keypair)
    try:
        receipt = ws_provider.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        print(
            "Extrinsic '{}' sent and included in block '{}'".format(
                receipt.extrinsic_hash, receipt.block_hash
            )
        )
        return jsonify(({
            "extrinsic_hash": receipt.extrinsic_hash,
            "block_hash": receipt.block_hash,
            "error": False,
            "message_error": ""
        })) 
    except SubstrateRequestException as e:
        print("Failed to send: {}".format(e))
        return jsonify(({
            "extrinsic_hash": "",
            "block_hash": "",
            "error": True,
            "message_error": "Failed to send: {}".format(e)
        }))


def get_revocation_registry_definition(id):
    log.info(f'Get revocation registry definition. Id: {id}')
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    rev_reg_def = ws_provider.query(
        module="Did",
        storage_function="RevocationRegistryDefinitions",
        params=[id]
    )
    if rev_reg_def == None:
        rev_reg_def_json = {}
    else:
        rev_reg_def_json = {
            "rev_reg_def_id": rev_reg_def.value["rev_reg_def_id"],
            "cred_def_id": rev_reg_def.value["cred_def_id"],
            "rev_reg_def_type": rev_reg_def.value["rev_reg_def_type"],
            "tag": rev_reg_def.value["tag"],
            "value": rev_reg_def.value["value"],
            "ver": rev_reg_def.value["ver"]
        }
        rev_reg_def_json["value"]["public_keys"] = json.load(rev_reg_def_json["value"]["public_keys"])
    return jsonify(({
        "revocation-registry-definition": rev_reg_def_json
    }))


def set_revocation_list():
    log.info('Set revocation list.')
    body = request.get_json()
    try:
        rev_list = body["rev_list"]
    except KeyError:
        return make_response(json.dumps({"message": "Bad request"}), status=400, mimetype="application/json")
    ws_provider = SubstrateInterface(f"ws://127.0.0.1:{common.config.config_service.config.node_http_rpc_port}")
    mnemonic = "bone laugh knife column endorse despair rail track lend hope tribe quote"
    keypair = Keypair.create_from_mnemonic(mnemonic, crypto_type=KeypairType.ED25519)

    if not "timestamp" in rev_list:
        rev_list["timestamp"] = None

    call = ws_provider.compose_call(
        call_module="Did",
        call_function="create_revocation_list",
        call_params={
            "rev_list": rev_list
        },
    )
    extrinsic = ws_provider.create_signed_extrinsic(call=call, keypair=keypair)
    try:
        receipt = ws_provider.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        print(
            "Extrinsic '{}' sent and included in block '{}'".format(
                receipt.extrinsic_hash, receipt.block_hash
            )
        )
        return jsonify(({
            "extrinsic_hash": receipt.extrinsic_hash,
            "block_hash": receipt.block_hash,
            "error": False,
            "message_error": ""
        })) 
    except SubstrateRequestException as e:
        print("Failed to send: {}".format(e))
        return jsonify(({
            "extrinsic_hash": "",
            "block_hash": "",
            "error": True,
            "message_error": "Failed to send: {}".format(e)
        }))