from utils import log
import requests
from Crypto import Random
import config


def get_psk() -> str:
    url = f"https://qrng.qbck.io/{config.config_service.current_config.qrng_api_key}/qbck/block/hex?size=1&length=32"
    try:
        response = requests.get(url)
        response.raise_for_status()
    except (requests.exceptions.RequestException, requests.exceptions.HTTPError):
        log.warning("Failed to get key from QRNG: Proceeding to fallback random psk...")
        return f"{Random.get_random_bytes(32).hex()}"
    else:
        data = response.json()
        return data['data']['result'][0]
