from typing import Optional

import requests
from common.config import config
from common.logger import log
from common import crypto

from .qkd import get_dec_key
from .qrng import generate_random_hex

PskWithSignature = tuple[str, str]
EncryptedPskResponse = tuple[str, str, str]


def generate_psk_from_qrng():
    log.info("Calling QRNG Api to get new PSK...")
    psk = generate_random_hex()
    log.debug(f"Generated psk: {psk}")

    return crypto.trim_0x_prefix(psk)


def get_psk_from_peers(psk_creator_peer_id: str = None) -> PskWithSignature:
    psks_with_sig = __fetch_from_peers()
    return __validate_psk(psks_with_sig, psk_creator_peer_id)


def __fetch_from_peers() -> [PskWithSignature]:
    log.info("Fetching PSK from other peers...")
    peers = config.config["peers"]

    psks_with_sig = []

    for peer_id, peer in peers.items():
        fetch_response = __fetch_encrypted_psk(peer_id, peer['server_addr'])
        if fetch_response is not None:
            encrypted_key, qkd_key_id, signature = fetch_response
            psk = __decrypt_psk(encrypted_key, peer["qkd_addr"], qkd_key_id)
            log.debug(f"Fetched psk: {psk} and signature: {signature}")
            psks_with_sig.append((psk, signature))

    return psks_with_sig


def __validate_psk(psks_with_sig: [PskWithSignature], psk_creator_peer_id: str = None) -> Optional[PskWithSignature]:
    # Keys and signatures from all the peers should be equal if we don't have psk creator peer id to verify against
    if psk_creator_peer_id is None:
        if len(set(psks_with_sig)) == 1:
            return psks_with_sig[0]
    # When we know psk creator peer id it's ok to get first verified key
    else:
        for psk, signature in psks_with_sig:
            if crypto.verify(psk, bytes.fromhex(signature), crypto.to_public_from_peerid(psk_creator_peer_id)):
                return psk, signature


def __fetch_encrypted_psk(peer_id: str, peer_addr: str) -> Optional[EncryptedPskResponse]:
    get_psk_url = f"{peer_addr}/peer/{config.config['local_peer_id']}/psk"
    get_psk_response = requests.get(get_psk_url)
    if get_psk_response.status_code != 200:
        log.error(f"{peer_id} didn't send the psk. Message: {get_psk_response.json()['message']}")
    else:
        response_body = get_psk_response.json()
        return response_body['key'], response_body['key_id'], response_body['signature']


def __decrypt_psk(encrypted_psk: str, qkd_addr: str, qkd_key_id: str) -> str:
    _, qkd_key = get_dec_key(qkd_addr, qkd_key_id)
    psk = crypto.xor(encrypted_psk, qkd_key)
    return crypto.trim_0x_prefix(psk)
