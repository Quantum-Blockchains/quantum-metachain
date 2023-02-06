from psk import fetch_from_peers
import config


config.config_service = config.ConfigService(config.Config())


def test_get_psk_from_peers(requests_mock):
    expected_psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    alice_server_addr = "http://localhost:5002"
    alice_qkd_addr = "http://localhost:9182/api/v1/keys/Alice1SAE"
    bob_peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    peers_response = {
        "key": "0x1f1205c6a4ac0e3ff341ad6ea8f2945d5fedbd86e1301e6f146e7358feaf5b02",
        "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "signature": "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    }
    qkd_reponse = {"keys": [{
        "key_ID": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "key": "LH8sve7mz7ifkzjgmu/jdVtdjkDbMynHmrId09b2Xd0="
    }]}

    get_psk_url = f"{alice_server_addr}/peer/{bob_peer_id}/psk"

    requests_mock.get(get_psk_url, json=peers_response)
    requests_mock.get(f"{alice_qkd_addr}/dec_keys?key_ID=ed1185e5-6223-415f-95fd-6364dcb2df32", json=qkd_reponse)

    result = fetch_from_peers("12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc")

    assert result == expected_psk
