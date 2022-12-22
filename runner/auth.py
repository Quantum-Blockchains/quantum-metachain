import logging

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.exceptions import InvalidSignature

# test
from config import config, abs_node_key_file_path
logging.basicConfig(format='%(asctime)s - %(message)s', level=logging.INFO)
# test


# Signs given data with ed25519 algorithm with private key from node_key path.
# Returns signature and saves it to path in argument
def sign(data, key_path: str, sig_path: str):
    try:
        private_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(key_path, 'rb').read())
        sig = private_key.sign(bytes(data, "utf-8")).hex()
        with open(sig_path, 'w') as file:
            file.write(sig)

    except Exception as e:
        logging.error(str(e))

    else:
        return sig


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

def verify_signature_agata(data: str, sig: str) -> bool:

    # test
    priv_key = ed25519.Ed25519PrivateKey.from_private_bytes(open(abs_node_key_file_path(), 'rb').read())
    pub_key = priv_key.public_key()

    public_bytes = pub_key.public_bytes(
        serialization.Encoding.OpenSSH,
        serialization.PublicFormat.OpenSSH
    )
    print(public_bytes)
    print(len(public_bytes))
    #test

    try:
        print(bytes(config["psk_creator_pub_key"], "utf-8"))
        print(len(bytes(config["psk_creator_pub_key"], "utf-8")))
        ## Public key from bytes to OpenSHH ?
        pub_key = ed25519.Ed25519PublicKey.from_public_bytes(bytes(config["psk_creator_pub_key"], "utf-8"))
        print(pub_key)

        pub_key.verify(bytes(sig, "utf-8"), bytes(data, "utf-8"))

    except InvalidSignature:
        logging.error("Invalid signature")

        return False

    except Exception as e:
        logging.error(e)

        return False

    else:
        return True
