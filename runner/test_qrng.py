from qrng import get_psk
from flask import jsonify
from config import config
from qkd import get_dec_key
from utils import xor

url = f"https://qrng.qbck.io/{config.config['qrng_api_key']}/qbck/block/hex?size=1&length=32"


def test_get_psk_from_qrng(requests_mock):
    expected_psk = "0xa9d6e6fd9b9fbdd2527b2b7919d0e19e2c5b64e9cb554760d8aa686c0131f282"
    qrng_response = {
        "data": {
            "result": [
                expected_psk
            ],
        }
    }
    requests_mock.get(url, json=qrng_response)

    result = get_psk()

    assert result == expected_psk


def test_get_psk_from_random(requests_mock):
    requests_mock.get(url, json={}, status_code=500)

    result = get_psk()

    assert len(result) == 64
    assert int(result, 16)


def test_get_psk_from_peers(requests_mock):
    expected_psk = "112fc0779ed82aad9fa668915d1f3eb89fa4a574fba346ff8004289f104d2279"
    alice_server_addr = "http://localhost:5002"
    alice_qkd_addr = "http://212.244.177.99:9182/api/v1/keys/Alice1SAE"
    bob_peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    peers_response = jsonify({
        "key": "0xbfc0991833c92497707ba7712678fc5b4e1443c5a5dc190a279a7cc84f16472b",
        "key_id": "80a2370f-e4da-490f-88a2-29281375255d",
        "signature": "35fdd5dfb8f0eeb3eebfa3eb26f98771854320f0651dc5f423457880904a0c66ceb5d8a55abef4ca946a919d3b8435481e4d1088daac4c0cc30b07fb2e29db0e"
    })
    get_psk_url = f"{alice_server_addr}/peer/{bob_peer_id}/psk"

    get_psk_response = requests_mock.get(get_psk_url, json=peers_response)
    response_body = get_psk_response.json()
    _, qkd_key = get_dec_key(alice_qkd_addr, response_body["key_id"])
    result = xor(response_body["key"], qkd_key)

    assert result == expected_psk
