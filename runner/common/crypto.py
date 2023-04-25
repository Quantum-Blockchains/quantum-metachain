import base64
import binascii

import base58
from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives._serialization import PublicFormat, Encoding
from cryptography.hazmat.primitives.asymmetric import ed25519

from .logger import log


def to_public(priv_key_str: str) -> bytes:
    priv_key_bytes = bytearray.fromhex(priv_key_str)
    private_key = ed25519.Ed25519PrivateKey.from_private_bytes(priv_key_bytes)
    return private_key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)


def to_public_from_peerid(peer_id: str) -> bytes:
    # hex[:12] is the Network ID
    peerid_hex = base58_to_hex(peer_id)[12:]
    return bytes.fromhex(peerid_hex)


def sign(data: bytes, priv_key_str: str) -> bytes:
    priv_key_bytes = bytearray.fromhex(priv_key_str)
    private_key = ed25519.Ed25519PrivateKey.from_private_bytes(priv_key_bytes)
    return private_key.sign(data)


def verify(data: bytes, signature: bytes, pub_key_bytes: bytes) -> bool:
    public_key = ed25519.Ed25519PublicKey.from_public_bytes(pub_key_bytes)
    try:
        public_key.verify(signature, data)
        return True
    except InvalidSignature:
        log.error("Invalid signature")
        return False


def base58_to_hex(val: str):
    return base58.b58decode(val).hex()


def base64_to_hex(message: str):
    return f"{base64.b64decode(message).hex()}"


def hex_to_base64(message: str):
    hex_bytes = binascii.unhexlify(message)
    base64_bytes = base64.b64encode(hex_bytes)
    return base64_bytes.decode('utf-8')


def is_hex(s):
    """
    Checks if the input string is a hexadecimal string.
    """
    try:
        int(s, 16)
        return True
    except ValueError:
        return False
