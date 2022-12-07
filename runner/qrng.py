import requests

from config import config


def get_psk():
    url = f"https://qrng.qbck.io/{config['qrng_api_key']}/qbck/block/hex?size=1&length=32"
    data = requests.get(url)
    data = data.json()
    return data['data']['result'][0]
