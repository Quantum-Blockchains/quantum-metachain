from psk import fetch_from_qrng


def test_qrng_response_successfull():
    resp = fetch_from_qrng()
    assert len(bytes.fromhex(resp)) == 32
