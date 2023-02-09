from unittest.mock import patch

import pytest

from common.crypto import xor
from web.external_server import get_psk, ExternalServerWrapper
import common.config
import common.file

psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
common.config.config_service = common.config.ConfigService(common.config.Config())
common.file.psk_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_file_path())
common.file.psk_sig_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_sig_file_path())


@patch('common.file.psk_file_manager.exists', return_value=True)
@patch('common.file.psk_sig_file_manager.exists', return_value=True)
@patch('common.file.psk_file_manager.read', return_value=psk)
@patch('common.file.psk_sig_file_manager.read', return_value=signature)
def test_get_psk_success(psk_sig_read, psk_read, psk_sig_exists, psk_exists, requests_mock):
    sig = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
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
        "signature": sig
    }


def test_get_psk_peer_config_missing():
    peer_id = "0000000000000000000000000000000000000000000000000"

    resp = get_psk(peer_id)

    assert resp.status_code == 404
    assert resp.get_json() == {"message": "Peer is not configured"}


def test_get_psk_psk_missing():
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    resp = get_psk(peer_id)

    assert resp.status_code == 404
    assert resp.get_json() == {"message": "Pre shared key not found"}


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
