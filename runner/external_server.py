from flask import Flask, request, make_response

import node
import psk_file
import logging
from config import settings

external_server = Flask(__name__)


@external_server.route("/psk", methods=['GET'])
def psk_get_key():
    psk_key = open(settings.PSK_FILE_PATH).read()
    body = request.get_json()
    qkd_key_id = body["key_ID"]
    qkd_key = body["key"]

    qkd_key_xor = xor(psk_key, qkd_key)

    logging.log(qkd_key, qkd_key_id, psk_key)
    logging.log(qkd_key_xor)

    return make_response()

def xor(a, b):
    xor_bin = [str(int(a) ^ int(b)) for a,b in zip(to_bin(a),to_bin(b))]
    xor_str = "%0*x" % ((len("".join(xor_bin)) + 3) // 4, int("".join(xor_bin), 2))
    return xor_str

def to_bin(str):
    return ''.join(format(ord(i), '0b') for i in str)
