from utils.crypto import sign
from config import config
from web.external_server import ExternalServerWrapper, get_psk
from utils import xor
from psk.psk_file import remove_psk_file, create_psk_file, exists_psk_file
from unittest.mock import patch


def prepare_psk():
    psk = "1234123412341234123412341234123412341234123412341234123412341234"
    sig = ""
    create_psk_file(psk)
    if not exists_psk_file():
        raise Exception("Cannot continue without psk")

    with open(config.abs_node_key_file_path()) as file:
        node_key = file.read()
        sig = sign(psk, node_key)

    with open(config.abs_psk_sig_file_path(), 'w') as file:
        file.write(sig.hex())

    return psk, sig


def test_get_psk_success(requests_mock):
    psk, sig = prepare_psk()
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    expected_resp = {
        "keys": [
            {
                "key_ID": "key_id",
                "key": "EjQ="  # encoded 1234
            }
        ]
    }
    expected_url = "http://localhost:9182/api/v1/keys/Alice1SAE/enc_keys?size=256"

    requests_mock.get(expected_url, json=expected_resp)

    wrapper = ExternalServerWrapper()

    with wrapper.external_server.app_context():
        resp = get_psk(peer_id)
        resp_body = resp.get_json()

    assert resp_body == {
        "key": xor(psk, "1234"),
        "key_id": "key_id",
        "signature": sig.hex()
    }


def test_get_psk_peer_config_missing():
    _, _ = prepare_psk()
    peer_id = "0000000000000000000000000000000000000000000000000"

    resp = get_psk(peer_id)

    assert resp.status_code == 404
    assert resp.get_json() == {"message": "Peer not found"}


def test_get_psk_psk_missing():
    try:
        remove_psk_file()
    except FileNotFoundError:
        print("file missing - continuing")

    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    resp = get_psk(peer_id)

    assert resp.status_code == 404
    assert resp.get_json() == {"message": "Couldn't find psk file"}


def test_get_psk_get_enc_key_failed(requests_mock):
    _, _ = prepare_psk()
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    expected_resp = {"error": "an error from QKD"}
    expected_url = "http://localhost:9182/api/v1/keys/Alice1SAE/enc_keys?size=256"

    requests_mock.get(expected_url, json=expected_resp)

    resp = get_psk(peer_id)

    assert resp.status_code == 500
    assert resp.get_json() == {"message": "Couldn't get enc key from QKD"}
