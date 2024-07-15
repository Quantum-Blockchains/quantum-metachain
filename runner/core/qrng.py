from common.logger import log
import requests
import validators
from Crypto import Random
import common.config
from urllib.parse import urlparse
from os import path


def generate_random_hex(length=32) -> str:
    # url = (f"https://qrng.qbck.io/{common.config.config_service.config.qrng_api_key}"
    #        f"/qbck/block/hex?size=1&length={length}")
    url = f"{common.config.config_service.config.qrng_url}/qrng/hex?size=32"
    if not validators.url(url):
        log.error("Invalid URL, please make sure that you have correct qRNG API key configured - proceeding to "
                  "fallback random psk...")
        return f"{Random.get_random_bytes(length).hex()}"
    url_pqkd = urlparse(common.config.config_service.config.local_qkd_url)
    try:
        if url_pqkd.scheme == "https":
            response = requests.get(url, cert=(path.join(common.config.node_dir, common.config.PQKD_CERT_PATH),
                                               path.join(common.config.node_dir, common.config.PQKD_KEY_PATH)),
                                    verify=path.join(common.config.node_dir, common.config.PQKD_VERIFY_PATH))
        else:
            response = requests.get(url)
        response.raise_for_status()
    except (requests.exceptions.RequestException, requests.exceptions.HTTPError):
        log.warning("Failed to get key from QRNG: Proceeding to fallback random psk...")
        return f"{Random.get_random_bytes(length).hex()}"
    else:
        data = response.json()
        # return data['data']['result'][0]
        return data['result']
