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

    logging.log(qkd_key, qkd_key_id, psk_key)

    return make_response()

