from requests.exceptions import InvalidURL
from core.qkd import get_enc_key, get_dec_key


def test_get_enc_key_success(requests_mock):
    enc_url = "http://correct.url"
    expected_url = f"{enc_url}/enc_keys?size=256"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(enc_url)
    assert key_id == "key_ID"
    assert decoded_key == "0x1234"


def test_get_enc_key_empty_url(requests_mock):
    enc_url = ""
    expected_url = f"{enc_url}/enc_keys?size=256"
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(enc_url)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_enc_key_invalid_url(requests_mock):
    enc_url = "http:/invalid schema"
    expected_url = f"{enc_url}/enc_keys?size=256"
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(enc_url)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_dec_key_success(requests_mock):
    key_id = "key_ID"
    dec_url = "http://correct.url"
    expected_url = f"{dec_url}/dec_keys?key_ID={key_id}"
    expected_resp = {
        "keys": [
            {
                "key_ID": key_id,
                "key": "QyE="  # encoded 4321
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id_response, decoded_key = get_dec_key(dec_url, key_id)
    assert key_id_response == key_id
    assert decoded_key == "0x4321"


def test_get_dec_key_empty_url(requests_mock):
    dec_url = ""
    expected_url = f"{dec_url}/dec_keys?key_ID="
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(dec_url)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_dec_key_invalid_url(requests_mock):
    dec_url = "http:/invalid schema"
    expected_url = f"{dec_url}/dec_keys?key_ID="
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(dec_url)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")
