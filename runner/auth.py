import logging

from cryptography.hazmat.primitives.asymmetric import ed25519

from config import abs_node_key_file_path


def sign(data):
    try:
        private_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(abs_node_key_file_path(), 'rb').read())
        signed_data = private_key.sign(data)

    except Exception as e:
        logging.error(str(e))

    return signed_data
