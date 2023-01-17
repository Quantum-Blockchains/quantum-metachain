import logging
import requests
from qkd import get_dec_key
from qrng import get_psk
from utils import xor
from crypto import verify, to_public_from_peerid
from config import config


def fetch_from_qrng():
    logging.info("Calling QRNG Api to get new PSK...")
    psk = get_psk()
    logging.debug(f"Generated psk: {psk}")

    return psk


def fetch_from_peers(peer_id):
    logging.info("Fetching PSK from other peers...")
    peers = config.config["peers"]

    psk = None
    while not psk:
        for peer in peers.values():
            get_psk_url = f"{peer['server_addr']}/peer/{config.config['local_peer_id']}/psk"
            get_psk_response = requests.get(get_psk_url)

            if get_psk_response.status_code != 200:
                logging.error(f"{peer_id} did'n send the psk. Message: {get_psk_response.json()['message']}")
            else:
                response_body = get_psk_response.json()
                _, qkd_key = get_dec_key(peer["qkd_addr"], response_body["key_id"])
                psk = xor(response_body["key"], qkd_key)
                signature = bytes.fromhex(response_body["signature"])
                if not verify(psk, signature, to_public_from_peerid(peer_id)):
                    logging.error("Couldn't verify psk signature")
                    # TODO JEQB-199 handle verification error

    logging.debug(f"Fetched psk {psk}")

    # TODO fetch psk from all peers in config, compare, verify and choose a valid one (TBD)
    return psk
