import base64
import binascii
import datetime

import base58
from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives._serialization import PublicFormat, Encoding, PrivateFormat
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa, ec
from cryptography.hazmat.backends import default_backend
from cryptography import x509
from cryptography.x509.oid import NameOID
from cryptography.hazmat.primitives import hashes

from .logger import log


def generate_self_signed_cert(country_name: str, state_name: str, locality_name: str, organization_name: str,
                              common_name: str, address: str) -> (str, str):
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
        backend=default_backend(),
    )
    subject = issuer = x509.Name([
        x509.NameAttribute(NameOID.COUNTRY_NAME, country_name),
        x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, state_name),
        x509.NameAttribute(NameOID.LOCALITY_NAME, locality_name),
        x509.NameAttribute(NameOID.ORGANIZATION_NAME, organization_name),
        x509.NameAttribute(NameOID.COMMON_NAME, common_name),
    ])
    cert = x509.CertificateBuilder().subject_name(
        subject
    ).issuer_name(
        issuer
    ).public_key(
        key.public_key()
    ).serial_number(
        x509.random_serial_number()
    ).not_valid_before(
        datetime.datetime.now(datetime.timezone.utc)
    ).not_valid_after(
        datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(days=365)
    ).add_extension(
        x509.SubjectAlternativeName([x509.DNSName(address)]),
        critical=False,
    ).sign(key, hashes.SHA256())
    cert_pem = cert.public_bytes(encoding=serialization.Encoding.PEM)
    key_pem = key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.TraditionalOpenSSL,
        encryption_algorithm=serialization.NoEncryption(),
    )
    return cert_pem, key_pem


def generate_ed25519() -> str:
    sk = ed25519.Ed25519PrivateKey.generate()
    return sk.private_bytes(Encoding.Raw, PrivateFormat.Raw, serialization.NoEncryption()).hex()


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
