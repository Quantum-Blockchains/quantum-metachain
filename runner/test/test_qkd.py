from requests.exceptions import InvalidURL

from core.qkd import get_enc_key, get_dec_key


def test_get_enc_key_http_success(requests_mock):
    qkd_config = {
        "url": "http://correct.url",
    }

    expected_url = f"{qkd_config['url']}/enc_keys?size=256"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(qkd_config)
    assert key_id == "key_ID"
    assert decoded_key == "1234"


def test_get_enc_key_https_success(requests_mock):
    qkd_config = {
        "url": "https://correct.url",
        "client_cert_path": "certificates/qbck-client.crt",
        "cert_key_path": "certificates/qbck-client.key",
    }
    expected_url = f"{qkd_config['url']}/enc_keys?size=256"

    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(qkd_config)
    assert key_id == "key_ID"
    assert decoded_key == "1234"
    assert requests_mock.last_request.scheme == 'https'
    assert requests_mock.last_request.verify is False
    assert requests_mock.last_request.cert == (qkd_config["client_cert_path"], qkd_config["cert_key_path"])


def test_get_enc_key_calls_without_cert_when_paths_are_not_passed_in_args(requests_mock):
    qkd_config = {
        "url": "http://correct.url",
        "client_cert_path": None,
        "cert_key_path": None,
    }
    expected_url = f"{qkd_config['url']}/enc_keys?size=256"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(qkd_config)
    assert key_id == "key_ID"
    assert decoded_key == "1234"
    assert requests_mock.last_request.verify is True
    assert requests_mock.last_request.cert is None


def test_get_enc_key_empty_url(requests_mock):
    qkd_config = {
        "url": "",

    }
    expected_url = f"{qkd_config['url']}/enc_keys?size=256"
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(qkd_config)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_enc_key_invalid_url(requests_mock):
    qkd_config = {
        "url": "http:/invalid schema",
    }
    expected_url = f"{qkd_config['url']}/enc_keys?size=256"
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_enc_key(qkd_config)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_dec_key_success(requests_mock):
    key_id = "key_ID"
    qkd_config = {
        "url": "http://correct.url",
    }
    expected_url = f"{qkd_config['url']}/dec_keys?key_ID={key_id}"
    expected_resp = {
        "keys": [
            {
                "key_ID": key_id,
                "key": "QyE="  # encoded 4321
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id_response, decoded_key = get_dec_key(key_id, qkd_config)
    assert key_id_response == key_id
    assert decoded_key == "4321"


def test_get_dec_key_https_success(requests_mock):
    key_id = "key_ID"

    qkd_config = {
        "url": "https://correct.url",
        "client_cert_path": "certificates/qbck-client.crt",
        "cert_key_path": "certificates/qbck-client.key",
    }
    expected_url = f"{qkd_config['url']}/dec_keys?key_ID={key_id}"
    expected_resp = {
        "keys": [
            {
                "key_ID": key_id,
                "key": "QyE="  # encoded 4321
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id_response, decoded_key = get_dec_key(key_id, qkd_config)
    assert key_id_response == key_id
    assert decoded_key == "4321"
    assert requests_mock.last_request.scheme == 'https'
    assert requests_mock.last_request.verify is False
    assert requests_mock.last_request.cert == (qkd_config["client_cert_path"], qkd_config["cert_key_path"])


def test_get_dec_key_calls_without_cert_when_paths_are_not_passed_in_args(requests_mock):
    key_id = "key_ID"

    qkd_config = {
        "url": "http://correct.url",
        "client_cert_path": None,
        "cert_key_path": None,
    }
    expected_url = f"{qkd_config['url']}/dec_keys?key_ID={key_id}"
    expected_resp = {
        "keys": [
            {
                "key_ID": key_id,
                "key": "QyE="  # encoded 4321
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id_response, decoded_key = get_dec_key(key_id, qkd_config)
    assert key_id_response == key_id
    assert decoded_key == "4321"
    assert requests_mock.last_request.scheme == 'http'
    assert requests_mock.last_request.verify is True
    assert requests_mock.last_request.cert is None


def test_get_dec_key_empty_url(requests_mock):
    qkd_config = {
        "url": "",
    }
    key_id = "key_ID"
    expected_url = f"{qkd_config['url']}/dec_keys?key_ID="
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_dec_key(key_id, qkd_config)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_get_dec_key_invalid_url(requests_mock):
    qkd_config = {
        "url": "http:/invalid schema",
    }
    key_id = "key_ID"

    expected_url = f"{qkd_config['url']}/dec_keys?key_ID="
    expected_resp = {}

    requests_mock.get(expected_url, json=expected_resp)

    try:
        _, _ = get_dec_key(key_id, qkd_config)
    except InvalidURL:
        return

    raise Exception("Expected error, didn't receive one")


def test_can_config_key_size(requests_mock):
    qkd_config = {
        "url": "http://correct.url",
        "key_size": 64
    }

    expected_url = f"{qkd_config['url']}/enc_keys?size=64"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(qkd_config)
    assert key_id == "key_ID"
    assert decoded_key == "1234"


def test_can_config_enc_key_response_format(requests_mock):
    qkd_config = {
        "url": "http://correct.url",
        "is_resp_base64": False
    }

    expected_url = f"{qkd_config['url']}/enc_keys?size=256"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_ID",
                "key": "1234"
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id, decoded_key = get_enc_key(qkd_config)
    assert key_id == "key_ID"
    assert decoded_key == "1234"


def test_can_config_dec_key_response_format(requests_mock):
    key_id = "key_ID"
    qkd_config = {
        "url": "http://correct.url",
        "is_resp_base64": False
    }

    expected_url = f"{qkd_config['url']}/dec_keys?key_ID={key_id}"
    expected_resp = {
        "keys": [
            {
                "key_ID": key_id,
                "key": "4321"
            }
        ]
    }
    requests_mock.get(expected_url, json=expected_resp)

    key_id_response, decoded_key = get_dec_key(key_id, qkd_config)
    assert key_id_response == key_id
    assert decoded_key == "4321"

