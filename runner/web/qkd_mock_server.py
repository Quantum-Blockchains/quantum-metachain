from flask import Flask, jsonify, request
from Crypto import Random

from common.crypto import hex_to_base64


class QkdMockServerWrapper:

    def __init__(self):
        self.qkd_mock_server = Flask(__name__)

        self.counter_id = 0
        self.keys = {}

        self.add_endpoint('/alice/bob/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/alice/bob/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/bob/alice/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/bob/alice/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/alice/dave/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/alice/dave/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/dave/alice/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/dave/alice/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/bob/charlie/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/bob/charlie/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/charlie/bob/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/charlie/bob/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/charlie/dave/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/charlie/dave/dec_keys', 'get_key', self.get_key, methods=['GET'])

        self.add_endpoint('/dave/charlie/enc_keys', 'generate_key', self.generate_key, methods=['GET'])
        self.add_endpoint('/dave/charlie/dec_keys', 'get_key', self.get_key, methods=['GET'])

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None, methods=['GET'], *args, **kwargs):
        self.qkd_mock_server.add_url_rule(endpoint, endpoint_name, handler, methods=methods, *args, **kwargs)

    def run(self):
        self.qkd_mock_server.run(None, 8182, False)

    def generate_key(self):
        args = request.args
        size = int(args.get('size'))
        key = Random.get_random_bytes(int(size / 8)).hex()
        key_base64 = hex_to_base64(key)

        self.keys[self.counter_id] = key_base64

        response = jsonify({
            "keys": [
                {
                    "key": key_base64,
                    "key_ID": self.counter_id
                }
            ]
        })

        self.counter_id = self.counter_id + 1

        return response

    def get_key(self):
        args = request.args
        key_id = int(args.get('key_ID'))
        response = jsonify({
            "keys": [
                {
                    "key": self.keys.get(key_id),
                    "key_ID": key_id
                }
            ]
        })

        self.keys.pop(key_id)

        return response
