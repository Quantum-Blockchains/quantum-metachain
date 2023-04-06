from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives._serialization import PublicFormat, Encoding
from cryptography.hazmat.primitives.asymmetric import ed25519
from .logger import log
import base64
import base58


def to_public(priv_key_str: str) -> bytes:
    priv_key_bytes = bytearray.fromhex(priv_key_str)
    private_key = ed25519.Ed25519PrivateKey.from_private_bytes(priv_key_bytes)
    return private_key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)


def to_public_from_peerid(peer_id: str) -> bytes:
    # hex[:12] is the Network ID
    peerid_hex = base58_to_hex(peer_id)[12:]
    return bytes.fromhex(peerid_hex)


def sign(data: str, priv_key_str: str) -> bytes:
    priv_key_bytes = bytearray.fromhex(priv_key_str)
    private_key = ed25519.Ed25519PrivateKey.from_private_bytes(priv_key_bytes)
    return private_key.sign(data.encode())


def verify(data: str, signature: bytes, pub_key_bytes: bytes) -> bool:
    public_key = ed25519.Ed25519PublicKey.from_public_bytes(pub_key_bytes)
    try:
        public_key.verify(signature, data.encode())
        return True
    except InvalidSignature:
        log.error("Invalid signature")
        return False


def base58_to_hex(val: str):
    return base58.b58decode(val).hex()


def base64_to_hex(message: str):
    return f"{base64.b64decode(message).hex()}"
