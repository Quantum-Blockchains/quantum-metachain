from common.logger import log
import requests
import validators
from Crypto import Random
from common.config import config


def generate_random_hex(length=32) -> str:
    url = f"https://qrng.qbck.io/{config.config['qrng_api_key']}/qbck/block/hex?size=1&length={length}"
    if not validators.url(url):
        log.error("Invalid URL, please make sure that you have correct qRNG API key configured - proceeding to "
                  "fallback random psk...")
        return f"{Random.get_random_bytes(length).hex()}"
    try:
        response = requests.get(url)
        response.raise_for_status()
    except (requests.exceptions.RequestException, requests.exceptions.HTTPError):
        log.warning("Failed to get key from QRNG: Proceeding to fallback random psk...")
        return f"{Random.get_random_bytes(length).hex()}"
    else:
        data = response.json()
        return data['data']['result'][0]
