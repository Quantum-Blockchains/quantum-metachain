from flask import Flask, request, jsonify, Response
from config import settings
from utils import base64_to_hex, to_hex, xor

import base64
import logging
import psk_file
import requests

external_server = Flask(__name__)


@external_server.route("/psk", methods=['GET'])
def psk_get_key():
    if not psk_file.exists():
        logging.error("Couldn't find psk file")
        return Response("{'error': 'Couldnt find psk file'}", status=422, mimetype="application/json")

    try:
        file = open(settings.PSK_FILE_PATH)
    except OSError:
        logging.error("Couldn't open psk file")
        return Response("{'error': 'Couldnt open psk file'}", status=500, mimetype="application/json")

    psk_key = file.read()

    body = request.get_json()
    peer_id = body["peer_id"]

    qkd_addr = str(settings.QKD_ADDR).split(",")
    qkd_url = ""
    cfg_peer_id = ""
    for addr in qkd_addr:
        split_addr = str(addr).split("/")
        cfg_peer_id = split_addr[len(split_addr) - 1]
        if peer_id == cfg_peer_id:
            cfg_peer_id = peer_id
            qkd_url += "http://"
            host_with_path = split_addr[2:len(split_addr) - 1]
            for part in host_with_path:
                qkd_url += part + "/"
            qkd_url += "enc_keys?size=256"

    if cfg_peer_id == "" or qkd_url == "":
        logging.error(f"{peer_id} not found - this peer is not configured")
        return Response("{'error': 'Peer not found'}", status=404, mimetype="application/json")

    logging.info(f"{qkd_url} - URL")
    qkd_resp = requests.get(qkd_url).json()
    key = qkd_resp["keys"][0]
    key_id = key["key_ID"]
    qkd_key = key["key"]

    decoded_qkd_key = base64_to_hex(qkd_key)

    xored_psk = xor(psk_key, decoded_qkd_key)

    return jsonify({
        "key": xored_psk,
        "key_id": key_id
    })

