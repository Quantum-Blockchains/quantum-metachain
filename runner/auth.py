import logging

from cryptography.hazmat.primitives.asymmetric import ed25519

from config import abs_node_key_file_path

def sign(file_abs_path):
    try:
        private_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(abs_node_key_file_path(), 'rb').read())

        signed_psk = private_key.sign(open(file_abs_path, 'rb').read())

        public_key = private_key.public_key()
        public_key.verify(signed_psk, open(file_abs_path, 'rb').read())

    except Exception as e:
        logging.error(str(e))

    return signed_psk
