import pickle
from dataclasses import dataclass
from typing import Optional

import requests

import common.config
from common import crypto
from common.logger import log
from core import onetimepad
from .qkd import get_dec_key
from .qrng import generate_random_hex

EncryptedPskResponse = tuple[str, str, str]


@dataclass(frozen=True)
class Psk:
    psk: str
    block_number: int = None
    signature: str = None

    def serialize(self):
        return pickle.dumps(self)


def generate_psk_from_qrng():
    log.info("Calling QRNG Api to get new PSK...")
    psk = generate_random_hex()
    log.debug(f"Generated psk: {psk}")

    return psk


def get_psk_from_peers(block_number: int = None, psk_creator_peer_id: str = None) -> Psk:
    psks_with_sig = __fetch_from_peers()
    return __validate_psk(psks_with_sig, block_number, psk_creator_peer_id)


def __fetch_from_peers() -> [Psk]:
    log.info("Fetching PSK from other peers...")
    peers = common.config.config_service.config.peers

    psks_with_sig = []

    for peer_id, peer_config in peers.items():
        fetch_response = __fetch_encrypted_psk(peer_id, peer_config['server_addr'])
        if fetch_response is not None:
            encrypted_key, qkd_key_id, signature = fetch_response
            psk = __decrypt_psk(encrypted_key, peer_config['qkd'], qkd_key_id)
            log.debug(f"Fetched psk: {psk} and signature: {signature}")
            psks_with_sig.append(Psk(psk, signature=signature))

    return psks_with_sig


def __validate_psk(psks_with_sig: [Psk], block_number: int = None, psk_creator_peer_id: str = None) -> Optional[Psk]:
    # Keys and signatures from all the peers should be equal if we don't have psk creator peer id to verify against
    if psk_creator_peer_id is None:
        if len(set(psks_with_sig)) == 1:
            return psks_with_sig[0]
        else:
            log.warning("Psk validation failed...")
    # When we know psk creator peer id it's ok to get first verified key
    else:
        for psk_obj in psks_with_sig:
            psk = psk_obj.psk
            signature = psk_obj.signature
            log.debug(f"validating psk: {psk}, signature: {signature}, psk_creator_peer_id: {psk_creator_peer_id}")
            psk_bytes = Psk(psk, block_number=block_number).serialize()
            if crypto.verify(psk_bytes, bytes.fromhex(signature),
                             crypto.to_public_from_peerid(psk_creator_peer_id)):
                return Psk(psk, signature=signature)
            else:
                log.warning("Psk validation failed...")


def __fetch_encrypted_psk(peer_id: str, peer_addr: str) -> Optional[EncryptedPskResponse]:
    get_psk_url = f"{peer_addr}/peer/{common.config.config_service.config.local_peer_id}/psk"
    get_psk_response = requests.get(get_psk_url)
    if get_psk_response.status_code != 200:
        log.error(f"{peer_id} didn't send the psk. Message: {get_psk_response.json()['message']}")
    else:
        response_body = get_psk_response.json()
        return response_body['key'], response_body['key_id'], response_body['signature']


def __decrypt_psk(encrypted_psk: str, qkd_config: dict, qkd_key_id: str) -> str:
    _, qkd_key = get_dec_key(qkd_key_id, qkd_config)
    psk = onetimepad.decrypt(encrypted_psk, qkd_key)
    return psk
