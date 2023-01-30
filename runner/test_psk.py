from psk import fetch_from_peers, validate_psk


def test_fetch_from_peers_get_results(requests_mock):
    expected_psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    expected_signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    alice_server_addr = "http://localhost:5002"
    alice_qkd_addr = "http://localhost:9182/api/v1/keys/Alice1SAE"
    bob_peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    peers_response = {
        "key": "0x1f1205c6a4ac0e3ff341ad6ea8f2945d5fedbd86e1301e6f146e7358feaf5b02",
        "key_id": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "signature": expected_signature
    }
    qkd_reponse = {"keys": [{
        "key_ID": "ed1185e5-6223-415f-95fd-6364dcb2df32",
        "key": "LH8sve7mz7ifkzjgmu/jdVtdjkDbMynHmrId09b2Xd0="
    }]}

    get_psk_url = f"{alice_server_addr}/peer/{bob_peer_id}/psk"

    requests_mock.get(get_psk_url, json=peers_response)
    requests_mock.get(f"{alice_qkd_addr}/dec_keys?key_ID=ed1185e5-6223-415f-95fd-6364dcb2df32", json=qkd_reponse)

    result = fetch_from_peers()

    assert result == [(expected_psk, expected_signature)]


def test_fetch_from_peers_returns_empty_list(requests_mock):
    alice_server_addr = "http://localhost:5002"
    bob_peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    peers_response = {
        "message": "Couldn't find psk file"
    }
    get_psk_url = f"{alice_server_addr}/peer/{bob_peer_id}/psk"

    requests_mock.get(get_psk_url, json=peers_response, status_code=404)

    result = fetch_from_peers()

    assert result == []


def test_validate_psk_without_creator_id_returns_psk_when_all_keys_from_peers_are_the_same():
    psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    psks_with_sig = [(psk, signature), (psk, signature), (psk, signature)]

    result = validate_psk(psks_with_sig)

    assert result == (psk, signature)


def test_validate_psk_without_creator_id_returns_none_when_not_all_keys_from_peers_are_the_same():
    psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    psks_with_sig = [(psk, signature), (psk, signature), ("invalid_psk", signature)]

    result = validate_psk(psks_with_sig)

    assert result is None


def test_validate_psk_with_creator_id_returns_psk_when_one_key_is_valid():
    psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    psks_with_sig = [("invalid_psk", signature), ("invalid_psk", signature), (psk, signature)]

    result = validate_psk(psks_with_sig, peer_id)

    assert result == (psk, signature)


def test_validate_psk_with_creator_id_returns_psk_when_one_signature_is_valid():
    psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"

    psks_with_sig = [(psk, "abcdef"), (psk, "abcdef"), (psk, signature)]

    result = validate_psk(psks_with_sig, peer_id)

    assert result == (psk, signature)


def test_validate_psk_with_creator_id_returns_none_when_keys_are_invalid():
    signature = "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"

    psks_with_sig = [("invalid_psk", signature), ("invalid_psk", signature), ("invalid_psk", signature)]

    result = validate_psk(psks_with_sig, peer_id)

    assert result is None


def test_validate_psk_with_creator_id_returns_none_when_signatures_are_invalid():
    psk = "336d297b4a4ac1876cd2958e321d772804b033c63a0337a88edc6e8b285906df"
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"

    psks_with_sig = [(psk, "abcdef"), (psk, "abcdef"), (psk, "abcdef")]

    result = validate_psk(psks_with_sig, peer_id)

    assert result is None
