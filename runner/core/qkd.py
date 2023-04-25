import requests
import validators

from common import crypto
from common.logger import log


def get_enc_key(url, cert_path=None, key_path=None, size=256):
    qkd_url = f"{url}/enc_keys?size={size}"
    if not validators.url(qkd_url):
        raise requests.exceptions.InvalidURL

    response = _call_qkd(qkd_url, cert_path, key_path)
    log.debug(f"response from qkd: {response}")
    return _unwrap_response(response)


def get_dec_key(url, key_id, cert_path=None, key_path=None):
    qkd_url = f"{url}/dec_keys?key_ID={key_id}"
    if not validators.url(qkd_url):
        raise requests.exceptions.InvalidURL

    response = _call_qkd(qkd_url, cert_path, key_path)
    log.debug(f"response from qkd: {response}")
    return _unwrap_response(response)


def _call_qkd(qkd_url, cert_path=None, key_path=None):
    if cert_path is None or key_path is None:
        return requests.get(qkd_url).json()
    else:
        return requests.get(qkd_url, cert=(cert_path, key_path), verify=False).json()


def _unwrap_response(response):
    key = response["keys"][0]
    key_id = key["key_ID"]
    qkd_key = key["key"]
    decoded_qkd_key = crypto.base64_to_hex(qkd_key)

    return key_id, decoded_qkd_key
