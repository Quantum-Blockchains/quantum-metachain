import requests
from cryptography.exceptions import InvalidSignature
import config
from .qkd import get_dec_key
from .qrng import get_psk
from utils import log, xor, trim_0x_prefix, verify, to_public_from_peerid


def fetch_from_qrng():
    log.info("Calling QRNG Api to get new PSK...")
    psk = get_psk()
    log.debug(f"Generated psk: {psk}")

    return psk


def fetch_from_peers(peer_id):
    log.info("Fetching PSK from other peers...")
    peers = config.config_service.current_config.peers

    psk = None
    while not psk:
        for peer in peers.values():
            get_psk_url = f"{peer['server_addr']}/peer/{config.config_service.current_config.local_peer_id}/psk"
            get_psk_response = requests.get(get_psk_url)

            if get_psk_response.status_code != 200:
                log.error(f"{peer_id} didn't send the psk. Message: {get_psk_response.json()['message']}")
            else:
                response_body = get_psk_response.json()
                _, qkd_key = get_dec_key(peer["qkd_addr"], response_body["key_id"])
                psk = xor(response_body["key"], qkd_key)
                psk = trim_0x_prefix(psk)
                signature = bytes.fromhex(response_body["signature"])
                if not verify(psk, signature, to_public_from_peerid(peer_id)):
                    log.error("Couldn't verify psk signature")
                    # TODO JEQB-199 handle verification error
                    raise InvalidSignature

    log.debug(f"Fetched psk {psk}")

    # TODO fetch psk from all peers in config, compare, verify and choose a valid one (TBD)
    return psk
