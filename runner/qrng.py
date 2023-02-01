import logging
import requests
import validators
from Crypto import Random
from config import config


def get_psk() -> str:
    url = f"https://qrng.qbck.io/{config.config['qrng_api_key']}/qbck/block/hex?size=1&length=32"
    if not validators.url(url):
        logging.error("Invalid URL, please make sure that you have correct qRNG API key configured - proceeding to "
                      "fallback random psk...")
        return f"{Random.get_random_bytes(32).hex()}"
    try:
        response = requests.get(url)
        response.raise_for_status()
    except (requests.exceptions.RequestException, requests.exceptions.HTTPError):
        logging.warning("Failed to get key from QRNG: Proceeding to fallback random psk...")
        return f"{Random.get_random_bytes(32).hex()}"
    else:
        data = response.json()
        return data['data']['result'][0]
