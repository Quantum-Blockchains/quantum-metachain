from flask import Flask, request, make_response

import node
import psk_file


external_server = Flask(__name__)


@external_server.route("/psk", methods=['GET'])
def psk_get_key():
    body = request.get_json()
    qkd_key_id = body["key_ID"]
    qkd_key = body["key"]

    return make_response()

