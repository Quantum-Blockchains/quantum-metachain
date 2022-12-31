import logging

import requests

from config import config
from qkd import get_dec_key
from qrng import get_psk
from utils import xor, base58_to_hex
from crypto import verify


def fetch_from_qrng():
    logging.info("Calling QRNG Api to get new PSK...")
    psk = get_psk()
    logging.debug(f"Generated psk: {psk}")

    return psk


def fetch_from_peers(peer_id):
    logging.info("Fetching PSK from other peers...")
    peers = config["peers"]

    psk = None
    while not psk:
        for p_id, peer in peers.items():
            get_psk_url = f"{peer['server_addr']}/peer/{config['local_peer_id']}/psk"
            get_psk_response = requests.get(get_psk_url)

            if get_psk_response.status_code != 200:
                logging.error(get_psk_response.json()["message"])
            else:
                response_body = get_psk_response.json()
                _, qkd_key = get_dec_key(peer["qkd_addr"], response_body["key_id"])
                psk = xor(response_body["key"], qkd_key)
                signature = bytes.fromhex(response_body["signature"])
                pub_key = bytes.fromhex(base58_to_hex(peer_id))
                if not verify(psk, signature, pub_key):
                    logging.error("Couldn't verify psk signature")
                    # TODO handle verification error

    logging.debug(f"Fetched psk {psk}")

    # TODO fetch psk from all peers in config, compare, verify and choose a valid one (TBD)
    return psk
