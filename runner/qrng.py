import requests

from config import settings


def get_psk():
    url = f"https://qrng.qbck.io/{settings.QRNG_API_KEY}/qbck/block/hex?size=1&length=32"
    data = requests.get(url)
    data = data.json()
    return data['data']['result'][0]
