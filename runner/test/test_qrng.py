from core.qrng import generate_random_hex
import common.config


url = f"https://qrng.qbck.io/{common.config.config_service.config.qrng_api_key}/qbck/block/hex?size=1&length=32"


def test_generate_random_hex_from_qrng(requests_mock):
    expected_psk = "0xa9d6e6fd9b9fbdd2527b2b7919d0e19e2c5b64e9cb554760d8aa686c0131f282"
    qrng_response = {
        "data": {
            "result": [
                expected_psk
            ],
        }
    }
    requests_mock.get(url, json=qrng_response)

    result = generate_random_hex()

    assert result == expected_psk


def test_generate_random_hex_from_local_entropy_when_qrng_fails(requests_mock):
    requests_mock.get(url, json={}, status_code=500)

    result = generate_random_hex()

    assert len(result) == 64
    assert int(result, 16)
