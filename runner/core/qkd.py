import requests
import validators

from common import crypto
from common.logger import log


def get_enc_key(qkd_config):
    qkd_url = f"{qkd_config['url']}/enc_keys?size={qkd_config.get('key_size', 256)}"
    if not validators.url(qkd_url):
        raise requests.exceptions.InvalidURL

    response = _call_qkd(qkd_url, qkd_config.get("client_cert_path"), qkd_config.get("cert_key_path"))
    log.debug(f"response from qkd: {response}")
    return __unwrap_response(response, qkd_config.get("is_resp_base64", True))


def get_dec_key(key_id, qkd_config):
    qkd_url = f"{qkd_config['url']}/dec_keys?key_ID={key_id}"
    if not validators.url(qkd_url):
        raise requests.exceptions.InvalidURL

    response = _call_qkd(qkd_url, qkd_config.get("client_cert_path"), qkd_config.get("cert_key_path"))
    log.debug(f"response from qkd: {response}")
    return __unwrap_response(response, qkd_config.get("is_resp_base64", True))


def _call_qkd(qkd_url, cert_path=None, key_path=None):
    if cert_path is None or key_path is None:
        return requests.get(qkd_url).json()
    else:
        return requests.get(qkd_url, cert=(cert_path, key_path), verify=False).json()


def __unwrap_response(response, is_base64):
    key = response["keys"][0]
    key_id = key["key_ID"]
    qkd_key = key["key"]

    if is_base64:
        qkd_key = crypto.base64_to_hex(qkd_key)

    return key_id, qkd_key
