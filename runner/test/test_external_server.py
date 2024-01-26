from unittest.mock import patch

import pytest

from common import exceptions
from core.onetimepad import encrypt
from web.external_server import get_psk, ExternalServerWrapper

psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
signature = ("17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1"
             "c7e5718651d076b430e9100")


@patch('common.file.psk_file_manager.exists', return_value=True)
@patch('common.file.psk_sig_file_manager.exists', return_value=True)
@patch('common.file.psk_file_manager.read', return_value=psk)
@patch('common.file.psk_sig_file_manager.read', return_value=signature)
def test_get_psk_success(psk_sig_read, psk_read, psk_sig_exists, psk_exists, requests_mock):
    sig = ("17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7"
           "e5718651d076b430e9100")
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    qkd_key = "qV4XorklC1EbehIbsovSaRGlWhyw3jETpt/laDSr3BQ="
    # a95e17a2b9250b511b7a121bb28bd26911a55a1cb0de3113a6dfe56834abdc14
    mocked_qkd_resp = {
        "keys": [
            {
                "key_ID": "key_id",
                "key": qkd_key
            }
        ]
    }
    expected_url = "http://localhost:9182/api/v1/keys/Alice1SAE/enc_keys?size=256"
    requests_mock.get(expected_url, json=mocked_qkd_resp)

    wrapper = ExternalServerWrapper()

    with wrapper.external_server.app_context():
        resp = get_psk(peer_id)
        resp_body = resp.get_json()

    assert resp_body == {
        "key": encrypt(psk, "a95e17a2b9250b511b7a121bb28bd26911a55a1cb0de3113a6dfe56834abdc14"),
        "key_id": "key_id",
        "signature": sig
    }


def test_get_psk_peer_config_missing():
    peer_id = "0000000000000000000000000000000000000000000000000"

    try:
        _ = get_psk(peer_id)
    except exceptions.PeerMisconfiguredError:
        return

    raise Exception("Test succeeded when it should fail")


def test_get_psk_psk_missing():
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    try:
        _ = get_psk(peer_id)
    except exceptions.PSKNotFoundError:
        return

    raise Exception("Test succeeded when it should fail")


# TODO remove this skip mark and test functionality after implementing error handler
@pytest.mark.skip(reason="global error handler for web not implemented yet")
def test_get_psk_get_enc_key_failed(requests_mock):
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    expected_resp = {"error": "an error from QKD"}
    expected_url = "http://localhost:9182/api/v1/keys/Alice1SAE/enc_keys?size=256"

    requests_mock.get(expected_url, json=expected_resp)

    resp = get_psk(peer_id)

    assert resp.status_code == 500
    assert resp.get_json() == {"message": "Couldn't get enc key from QKD"}
