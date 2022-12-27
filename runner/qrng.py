import logging
import requests
from Crypto import Random


def get_psk(config) -> str:
    url = f"https://qrng.qbck.io/{config['qrng_api_key']}/qbck/block/hex?size=1&length=32"
    try:
        response = requests.get(url)
        response.raise_for_status()
    except (requests.exceptions.RequestException, requests.exceptions.HTTPError):
        logging.warning("Failed to get key from QRNG: Proceeding to fallback random psk...")
        return f"0x{Random.get_random_bytes(32).hex()}"
    else:
        data = response.json()
        return data['data']['result'][0]
