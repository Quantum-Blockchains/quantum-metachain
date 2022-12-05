from flask import Flask, request, make_response, jsonify

import node
import psk_file
import string
import requests
import logging
from config import settings

external_server = Flask(__name__)


@external_server.route("/psk", methods=['GET'])
def psk_get_key():
    psk_key = open(settings.PSK_FILE_PATH).read()
    body = request.get_json()
    peer_id = body["peer_id"]
    logging.info(f"started request")

    qkd_addr = str(settings.QKD_ADDR).split(",")
    qkd_url = ""
    cfg_peer_id = ""
    for addr in qkd_addr:
        split_addr = str(addr).split("/")
        logging.info(f"split address: {split_addr} ")
        cfg_peer_id = split_addr[len(split_addr)-1]
        if peer_id == cfg_peer_id:
            logging.info(f"matched {peer_id} peer!")
            cfg_peer_id = peer_id
            qkd_url += "http://"
            host_with_path = split_addr[2:len(split_addr)-1]
            for part in host_with_path:
                qkd_url += part + "/"
            qkd_url += "enc_keys?size=256"
            logging.info(f"Resulting QKD URL: {qkd_url}")

    if cfg_peer_id == "" or qkd_url == "":
        logging.info(f"{peer_id} not found - this peer is not configured")
        return make_response()

    qkd_resp = requests.get(qkd_url).json()
    keys = qkd_resp["keys"]
    key_id = keys["key_ID"]
    qkd_key = keys["key"]

    logging.info(f"psk key: {psk_key}")

    xored = xor(psk_key, qkd_key)

    return jsonify({
        "key": xored,
        "key_id": key_id
    })


def xor(a, b):
    xor_bin = [str(int(a) ^ int(b)) for a,b in zip(to_bin(a),to_bin(b))]
    xor_str = "%0*x" % ((len("".join(xor_bin)) + 3) // 4, int("".join(xor_bin), 2))
    return xor_str


def to_bin(str):
    return ''.join(format(ord(i), '0b') for i in str)
