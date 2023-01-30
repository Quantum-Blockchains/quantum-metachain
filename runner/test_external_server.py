from external_server import get_psk
from psk_file import create, exists
from crypto import sign
from config import config
from external_server import ExternalServerWrapper
from utils import xor
from psk_file import remove


def prepare_psk():
    psk = "1234123412341234123412341234123412341234123412341234123412341234"
    sig = ""
    create(psk)
    if not exists():
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


def test_get_psk_psk_missing():
    try:
        remove()
    except FileNotFoundError:
        print("file missing - continuing")

    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    resp = get_psk(peer_id)

    assert resp.status_code == 404
