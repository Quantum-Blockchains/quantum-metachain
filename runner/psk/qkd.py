import requests
from utils import base64_to_hex


def get_enc_key(url):
    qkd_url = f"{url}/enc_keys?size=256"
    response = requests.get(qkd_url).json()
    return _unwrap_response(response)


def get_dec_key(url, key_id):
    qkd_url = f"{url}/dec_keys?key_ID={key_id}"
    response = requests.get(qkd_url).json()

    return _unwrap_response(response)


def _unwrap_response(response):
    key = response["keys"][0]
    key_id = key["key_ID"]
    qkd_key = key["key"]
    decoded_qkd_key = base64_to_hex(qkd_key)

    return key_id, decoded_qkd_key
