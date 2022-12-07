import logging

import requests

from config import config
from qkd import get_dec_key
from qrng import get_psk
from utils import xor


def fetch_from_qrng():
    logging.info("Calling QRNG Api to get new PSK...")
    psk = get_psk()
    logging.debug(f"Generated psk: {psk}")

    return psk


def fetch_from_peers():
    logging.info("Fetching PSK from other peers...")
    peers = config["peers"]
    for peer_id, peer in peers.items():
        get_psk_url = f"{peer['server_addr']}/peer/{peer_id}/psk"
        get_psk_response = requests.get(get_psk_url).json()
        _, qkd_key = get_dec_key(peer["qkd_addr"], get_psk_response["key_id"])
        psk = xor(get_psk_response["key"], qkd_key)
        logging.debug(f"Fetched psk {psk}")

        # TODO fetch psk from all peers in config, compare, verify and choose a valid one (TBD)
        return psk
