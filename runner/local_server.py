from flask import Flask, request, make_response

import psk_file
from config import settings

app = Flask(__name__)


@app.route("/psk", methods=['POST'])
def save_pre_shared_key():
    body = request.get_json()
    key = body["psk"]

    if is_valid_hex(key):
        psk_file.create(key)

    return make_response()


def is_valid_hex(psk):
    return psk and int(psk, 16) and len(psk) == 64


if __name__ == '__main__':
    app.run(port=settings.LOCAL_SERVER_PORT)
