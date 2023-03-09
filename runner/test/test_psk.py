import pytest

import common.config
from core.pre_shared_key import get_psk_from_peers


@pytest.fixture()
def before_each(requests_mock):
    common.config.config_service.current_config.peers = {
        "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc": {
            "qkd_addr": "http://localhost:9182",
            "server_addr": "http://localhost:5002"
        },
        "12D3KooWAmo51HBQgCnFhTbGQD47swoBqiBvKYk6fXCJQJfuhhaY": {
            "qkd_addr": "http://localhost:9182",
            "server_addr": "http://localhost:5001"
        }
    }
    alice_qkd_addr = "http://localhost:9182"
    qkd_reponse = {"keys": [{
        "key_ID": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "key": "LH8sve7mz7ifkzjgmu/jdVtdjkDbMynHmrId09b2Xd0="
    }]}
    requests_mock.get(f"{alice_qkd_addr}/dec_keys?key_ID=ed1185e5-6223-415f-95fd-6364dcb2df32", json=qkd_reponse)

    yield


def test_get_psk_without_creator_id_returns_psk_when_all_keys_from_peers_are_the_same(requests_mock, before_each):
    encrypted_key = "01500102005a55065104515700575a0f0f055d010a0d5d550a5354025204050d055657540b56570657030300010e020f0104065107015c51560e530f05520002"
    expected_psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    __create_alice_peer_response_mock(requests_mock, encrypted_key, expected_signature)
    __create_bob_peer_response_mock(requests_mock, encrypted_key, expected_signature)

    result = get_psk_from_peers()

    assert result == (expected_psk, expected_signature)


def test_get_psk_without_creator_id_returns_none_when_not_all_keys_from_peers_are_the_same(requests_mock, before_each):
    encrypted_key = "01500102005a55065104515700575a0f0f055d010a0d5d550a5354025204050d055657540b56570657030300010e020f0104065107015c51560e530f05520002"
    invalid_encrypted_key = "00480253075602535254555555065356520809560903030855000951555655030705055205540855040054520303020953070951520201545403540656060554"  # encrypted 0x0...0
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    __create_alice_peer_response_mock(requests_mock, encrypted_key, expected_signature)
    __create_bob_peer_response_mock(requests_mock, invalid_encrypted_key, expected_signature)

    result = get_psk_from_peers()

    assert result is None


def test_get_psk_with_creator_id_returns_psk_when_one_key_is_valid(requests_mock, before_each):
    encrypted_key = "01500102005a55065104515700575a0f0f055d010a0d5d550a5354025204050d055657540b56570657030300010e020f0104065107015c51560e530f05520002"
    invalid_encrypted_key = "00480253075602535254555555065356520809560903030855000951555655030705055205540855040054520303020953070951520201545403540656060554"
    expected_key = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    __create_alice_peer_response_mock(requests_mock, encrypted_key, expected_signature)
    __create_bob_peer_response_mock(requests_mock, invalid_encrypted_key, expected_signature)

    result = get_psk_from_peers(peer_id)

    assert result == (expected_key, expected_signature)


def test_get_psk_with_creator_id_returns_psk_when_one_signature_is_valid(requests_mock, before_each):
    encrypted_key = "01500102005a55065104515700575a0f0f055d010a0d5d550a5354025204050d055657540b56570657030300010e020f0104065107015c51560e530f05520002"
    expected_key = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    invalid_signature = "000000"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    __create_alice_peer_response_mock(requests_mock, encrypted_key, invalid_signature)
    __create_bob_peer_response_mock(requests_mock, encrypted_key, expected_signature)

    result = get_psk_from_peers(peer_id)

    assert result == (expected_key, expected_signature)


def test_get_psk_with_creator_id_returns_none_when_all_keys_are_invalid(requests_mock, before_each):
    invalid_encrypted_key = "00480253075602535254555555065356520809560903030855000951555655030705055205540855040054520303020953070951520201545403540656060554"
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    __create_alice_peer_response_mock(requests_mock, invalid_encrypted_key, expected_signature)
    __create_bob_peer_response_mock(requests_mock, invalid_encrypted_key, expected_signature)

    result = get_psk_from_peers(peer_id)

    assert result is None


def test_get_psk_with_creator_id_returns_none_when_all_signatures_are_invalid(requests_mock, before_each):
    encrypted_key = "01500102005a55065104515700575a0f0f055d010a0d5d550a5354025204050d055657540b56570657030300010e020f0104065107015c51560e530f05520002"
    invalid_signature = "000000"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    __create_alice_peer_response_mock(requests_mock, encrypted_key, invalid_signature)
    __create_bob_peer_response_mock(requests_mock, encrypted_key, invalid_signature)

    result = get_psk_from_peers(peer_id)

    assert result is None


def test_get_psk_returns_none_when_peers_response_with_an_error(requests_mock):
    peers_response = {
        "message": "Couldn't find psk file"
    }
    alice_psk_url = "http://localhost:5002/peer/12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW/psk"
    bob_psk_url = "http://localhost:5001/peer/12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW/psk"

    requests_mock.get(alice_psk_url, json=peers_response, status_code=404)
    requests_mock.get(bob_psk_url, json=peers_response, status_code=404)

    result = get_psk_from_peers()

    assert result is None


def __create_alice_peer_response_mock(requests_mock, encrypted_key, signature):
    alice_server_addr = "http://localhost:5002"
    peers_response = {
        "key": encrypted_key,
        "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "signature": signature
    }

    get_psk_url = f"{alice_server_addr}/peer/12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW/psk"
    requests_mock.get(get_psk_url, json=peers_response)


def __create_bob_peer_response_mock(requests_mock, encrypted_key, signature):
    bob_server_addr = "http://localhost:5001"
    peers_response = {
        "key": encrypted_key,
        "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "signature": signature
    }

    get_psk_url = f"{bob_server_addr}/peer/12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW/psk"
    requests_mock.get(get_psk_url, json=peers_response)
