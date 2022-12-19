import logging

from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.exceptions import InvalidSignature


# Signs given data with ed25519 algorithm with private key from node_key path.
# Returns signature and saves it to path in argument
def sign(data, key_path: str, sig_path: str):
    try:
        private_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(key_path, 'rb').read())
        signed_data = private_key.sign(bytes(data, "utf-8")).hex()
        with open(sig_path, 'w') as file:
            file.write(signed_data)

    except Exception as e:
        logging.error(str(e))

    else:
        return signed_data


# Verifies signature generated with given ed25519 private key and given input.
# Both arguments are absolute paths to said parameters
def verify_signature(data: str, sig_path: str, node_key_path: str) -> bool:
    try:
        with open(sig_path, "r") as sig_file:
            signature = sig_file.read()

        priv_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(node_key_path, 'rb').read())

    except Exception as e:
        logging.error(str(e))

        return False

    pub_key = priv_key.public_key()
    try:
        pub_key.verify(bytes(signature, "utf-8"), bytes(data, "utf-8"))

    except InvalidSignature:
        logging.error("Invalid signature")

        return False

    except Exception as e:
        logging.error(e)

        return False

    else:
        return True
